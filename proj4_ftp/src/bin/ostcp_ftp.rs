use proj4_ftp::cli_util::{cmd_prompt, flush_stdout, getline, pick, resp_prompt};
use proj4_ftp::ftp_cmds::FtpCmd;

use std::io::{BufRead, BufReader, Read, Result, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};

/// FTP server response: status code, original message
type FtpResp = (u32, String);
/// FTP server response, passive data received, should exit
type FtpHandleResult = (FtpResp, String, bool);

/// read all data from a TCP connection
fn read_all(stream: &mut TcpStream) -> Vec<u8> {
  let mut data = vec![];
  stream.read_to_end(&mut data).unwrap();
  data
}

/// FTP client object:msg handles authentication, control message and data transmission.
pub struct FtpClient {
  server_addr: Ipv4Addr,
  ctrl_stream: TcpStream,
  resp_stream: BufReader<TcpStream>,
  pasv_port: Option<u16>,
}
impl FtpClient {
  /// get the response from server
  fn read_response(&mut self) -> FtpResp {
    fn continued(line: &str) -> bool {
      line.as_bytes()[3] == b'-'
    }
    let mut raw_resp = String::new();
    let mut line = String::new();
    self.resp_stream.read_line(&mut line).unwrap();
    raw_resp.push_str(&line);

    let status = line[..3].parse().unwrap();
    log::debug!("response status {status}");

    if continued(&line) {
      loop {
        line.clear();
        self.resp_stream.read_line(&mut line).unwrap();
        raw_resp.push_str(&line);
        if !continued(&line) {
          break;
        }
      }
    }
    log::debug!("raw response text:\n{raw_resp}");

    (status, raw_resp)
  }
  /// send a command for the server.
  pub fn send_raw_command(&mut self, mut cmd: String) -> FtpResp {
    log::debug!("send command {cmd}");
    cmd.push_str("\r\n");
    self.ctrl_stream.write_all(cmd.as_bytes()).unwrap();
    self.read_response()
  }
  /// send a structured ftp command for the server
  pub fn send_ftp_command(&mut self, cmd: FtpCmd) -> FtpResp {
    self.send_raw_command(cmd.as_ftp_request())
  }

  /// connect to FTP server, retrive the welcome message
  pub fn connect(server_addr: SocketAddrV4) -> Result<(Self, FtpResp)> {
    log::debug!("connecting to {:?}", server_addr);
    let stream = TcpStream::connect(server_addr)?;
    let mut this = Self {
      server_addr: *server_addr.ip(),
      ctrl_stream: stream.try_clone().unwrap(),
      resp_stream: BufReader::new(stream),
      pasv_port: None,
    };
    log::debug!("connected");
    let welcome = this.read_response();
    Ok((this, welcome))
  }

  /// send PASV and get passive data port
  fn pasv(&mut self) -> FtpResp {
    let resp = self.send_ftp_command(FtpCmd::PASV);
    let (status, msg) = resp.clone();
    if status != 227 {
      return resp;
    }

    let l = msg.find('(').unwrap();
    let r = msg.find(')').unwrap();
    // (host: 4, port: 2)
    let nums: Vec<u16> = msg[l + 1..r].split(',').map(|x| x.parse::<u16>().unwrap()).collect();
    let n = nums.len();

    let port = nums[n - 2] << 8 | nums[n - 1];
    log::debug!("passive mode port {port}");
    self.pasv_port = Some(port);

    (status, msg)
  }

  fn conn_pasv(&mut self) -> Option<TcpStream> {
    let port = self.pasv_port.take()?;
    log::debug!("connect to passive port {port}");
    TcpStream::connect(SocketAddrV4::new(self.server_addr, port)).ok()
  }

  /// handle one ftp command.
  /// return the 3-tuple of
  /// (Ftp server response, passive data transmitted, should exit)
  pub fn handle_ftp(&mut self, cmd: FtpCmd) -> FtpHandleResult {
    log::debug!("handling command {:?}", cmd);
    match cmd.clone() {
      FtpCmd::LIST(_) => {
        if let Some(mut conn) = self.conn_pasv() {
          let resp = self.send_ftp_command(cmd);
          if resp.0 != 150 {
            return (resp, "".into(), false);
          }
          let list = String::from_utf8(read_all(&mut conn)).unwrap();
          let resp = self.read_response();
          (resp, list, false)
        } else {
          let resp = self.send_ftp_command(cmd);
          (resp, "".into(), false)
        }
      }
      FtpCmd::RETR(file) => {
        if let Some(mut conn) = self.conn_pasv() {
          let resp = self.send_ftp_command(cmd);
          if resp.0 != 150 {
            return (resp, "".into(), false);
          }
          std::fs::write(file, read_all(&mut conn)).unwrap();
          let resp = self.read_response();
          (resp, "".into(), false)
        } else {
          let resp = self.send_ftp_command(cmd);
          (resp, "".into(), false)
        }
      }
      FtpCmd::PASV => {
        let resp = self.pasv();
        (resp, "".into(), false)
      }
      FtpCmd::QUIT => {
        let resp = self.send_ftp_command(cmd);
        (resp, "".into(), true)
      }
      _ => {
        let resp = self.send_ftp_command(cmd);
        (resp, "".into(), false)
      }
    }
  }
}

fn main() -> Result<()> {
  env_logger::init();

  // bftpd server listening at localhost:21
  // let server_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 21);
  let server_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 21);
  let (mut ftp, welcome) = FtpClient::connect(server_addr)?;
  println!("{}", resp_prompt(welcome.1));

  loop {
    print!("{} ", cmd_prompt("cmd>"));
    flush_stdout().unwrap();
    let cmds = FtpCmd::parse(&getline()?);

    let cmd = cmds
      .first()
      .filter(|x| x.is_exact())
      .cloned()
      .or_else(|| pick(&cmds))
      .map(|x| x.cmd());

    if let Some(cmd) = cmd {
      let ((status, resp), passive_data, should_exit) = ftp.handle_ftp(cmd);
      println!("{} {}", resp_prompt(format!("resp[{status}]")), resp);
      println!("{passive_data}");
      if should_exit {
        std::process::exit(0)
      }
    }
  }
}
