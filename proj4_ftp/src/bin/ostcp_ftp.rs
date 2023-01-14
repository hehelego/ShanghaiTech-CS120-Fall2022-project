use console::{style, StyledObject};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::io::{stdin, stdout};
use std::io::{ErrorKind, Read, Result, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::Duration;

/// FTP communication buffer size, also the maximum command line length
const BUF_SZ: usize = 4096;
type FtpResp = (u32, String);
type FtpHandleResult = (FtpResp, String, bool);

fn read_to_end(stream: &mut TcpStream) -> Vec<u8> {
  let mut buf = [0; BUF_SZ];
  let mut n = 0;

  loop {
    match stream.read(&mut buf[n..]) {
      Ok(m) if m > 0 => {
        n += m;
      }
      Err(e) if e.kind() == ErrorKind::Interrupted => {}
      _ => {
        break;
      }
    };
  }

  buf[..n].to_vec()
}

fn match_metric(pattern: &str, text: &str) -> f32 {
  let matches = pattern.chars().zip(text.chars()).take_while(|(x, y)| x == y).count();
  matches as f32 / pattern.len() as f32
}

#[derive(Debug, Clone)]
pub enum FtpCmd {
  USER(String),
  PASS(String),
  PWD,
  CWD(String),
  PASV,
  LIST(String),
  RETR(String),
  QUIT,
}

#[derive(Debug, Clone)]
pub struct FtpCmdFuzz {
  matched_cmd: FtpCmd,
  match_rate: f32,
}

impl FtpCmdFuzz {
  fn new(matched_cmd: FtpCmd, match_rate: f32) -> Self {
    Self {
      matched_cmd,
      match_rate,
    }
  }
  fn some_new(matched_cmd: FtpCmd, match_rate: f32) -> Option<Self> {
    Some(Self::new(matched_cmd, match_rate))
  }

  fn cmp(a: &Self, b: &Self) -> Ordering {
    PartialOrd::partial_cmp(&a.match_rate, &b.match_rate).unwrap_or(Ordering::Equal)
  }

  pub fn is_exact(&self) -> bool {
    self.match_rate == 1.0
  }

  pub fn cmd(&self) -> FtpCmd {
    self.matched_cmd.clone()
  }
}

impl FtpCmd {
  fn as_ftp_request(&self) -> String {
    match self {
      FtpCmd::USER(name) => format!("USER {name}"),
      FtpCmd::PASS(password) => format!("PASS {password}"),
      FtpCmd::PWD => "PWD".into(),
      FtpCmd::CWD(dir) => format!("CWD {dir}"),
      FtpCmd::PASV => "PASV".into(),
      FtpCmd::LIST(dir) => {
        let sep = if dir.is_empty() { "" } else { " " };
        format!("LIST{sep}{dir}")
      }
      FtpCmd::RETR(file) => format!("RETR {file}"),
      FtpCmd::QUIT => "QUIT".into(),
    }
  }

  fn parse_user(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    let (cmd, uname) = raw_cmd.split_once(" ")?;
    FtpCmdFuzz::some_new(FtpCmd::USER(uname.into()), match_metric("USER", cmd))
  }
  fn parse_pass(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    if let Some((cmd, pwd)) = raw_cmd.split_once(" ") {
      FtpCmdFuzz::some_new(FtpCmd::PASS(pwd.into()), match_metric("PASS", cmd))
    } else {
      FtpCmdFuzz::some_new(FtpCmd::PASS("".into()), match_metric("PASS", raw_cmd))
    }
  }
  fn parse_cwd(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    let (cmd, dir) = raw_cmd.split_once(" ")?;
    FtpCmdFuzz::some_new(FtpCmd::CWD(dir.into()), match_metric("CWD", cmd))
  }
  fn parse_list(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    if let Some((cmd, dir)) = raw_cmd.split_once(" ") {
      FtpCmdFuzz::some_new(FtpCmd::LIST(dir.into()), match_metric("LIST", cmd))
    } else {
      FtpCmdFuzz::some_new(FtpCmd::LIST("".into()), match_metric("LIST", raw_cmd))
    }
  }
  fn parse_retr(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    let (cmd, file) = raw_cmd.split_once(" ")?;
    FtpCmdFuzz::some_new(FtpCmd::RETR(file.to_string()), match_metric("RETR", cmd))
  }
  fn parse_pwd(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    FtpCmdFuzz::some_new(FtpCmd::PWD, match_metric("PWD", raw_cmd))
  }
  fn parse_pasv(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    FtpCmdFuzz::some_new(FtpCmd::PASV, match_metric("PASV", raw_cmd))
  }
  fn parse_quit(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    FtpCmdFuzz::some_new(FtpCmd::QUIT, match_metric("QUIT", raw_cmd))
  }

  pub fn parse(raw_cmd: &str) -> Vec<FtpCmdFuzz> {
    let mut ret = vec![];
    fn add_one<F: Fn(&str) -> Option<FtpCmdFuzz>>(parse_func: F, cmd: &str, ret: &mut Vec<FtpCmdFuzz>) {
      if let Some(parse_ret) = parse_func(cmd).filter(|x| x.match_rate != 0.0) {
        ret.push(parse_ret);
      }
    }
    let raw_cmd = raw_cmd.trim();
    add_one(Self::parse_user, raw_cmd, &mut ret);
    add_one(Self::parse_pass, raw_cmd, &mut ret);
    add_one(Self::parse_pwd, raw_cmd, &mut ret);
    add_one(Self::parse_cwd, raw_cmd, &mut ret);
    add_one(Self::parse_pasv, raw_cmd, &mut ret);
    add_one(Self::parse_list, raw_cmd, &mut ret);
    add_one(Self::parse_retr, raw_cmd, &mut ret);
    add_one(Self::parse_quit, raw_cmd, &mut ret);

    ret.sort_by(FtpCmdFuzz::cmp);
    ret.reverse();
    ret
  }
}

/// FTP client object:msg handles authentication, control message and data transmission.
pub struct FtpClient {
  server_addr: Ipv4Addr,
  ctrl_stream: TcpStream,
  pasv_port: Option<u16>,
}
impl FtpClient {
  /// get the response from server
  fn read_response(&mut self) -> FtpResp {
    let resp = read_to_end(&mut self.ctrl_stream);
    log::debug!("raw response from the server:\n{:?}", resp);
    let resp = String::from_utf8(resp).unwrap();
    let pos = resp.trim().find(' ').unwrap();
    let status = &resp[..pos];
    log::debug!("response status={status}: with message={resp}");
    (status.parse().unwrap(), resp.into())
  }
  /// send a command for the server, return the response
  pub fn send_raw_command(&mut self, mut cmd: String) -> FtpResp {
    log::debug!("send command {cmd}");
    cmd.push_str("\r\n");
    self.ctrl_stream.write_all(cmd.as_bytes()).unwrap();
    self.read_response()
  }
  /// send a command for the server, return the response
  pub fn send_ftp_command(&mut self, cmd: FtpCmd) -> FtpResp {
    self.send_raw_command(cmd.as_ftp_request())
  }

  /// connect to FTP server, retrive the welcome message
  pub fn connect(server_addr: SocketAddrV4) -> Result<(Self, FtpResp)> {
    log::debug!("connecting to {:?}", server_addr);
    let ctrl_stream = TcpStream::connect(server_addr)?;
    ctrl_stream.set_read_timeout(Some(Duration::from_millis(700))).unwrap();
    let mut this = Self {
      server_addr: *server_addr.ip(),
      ctrl_stream,
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
    match cmd {
      FtpCmd::LIST(_) => {
        if let Some(mut conn) = self.conn_pasv() {
          let resp = self.send_ftp_command(cmd);
          let list = read_to_end(&mut conn);
          (resp, String::from_utf8(list).unwrap(), false)
        } else {
          let resp = self.send_ftp_command(cmd);
          (resp, "".into(), false)
        }
      }
      FtpCmd::RETR(file) => {
        if let Some(mut conn) = self.conn_pasv() {
          let resp = self.send_ftp_command(FtpCmd::RETR(file.clone()));
          std::fs::write(file, read_to_end(&mut conn)).unwrap();
          (resp, "".into(), false)
        } else {
          let resp = self.send_ftp_command(FtpCmd::RETR(file.clone()));
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

fn cmd_prompt<D>(prompt: D) -> StyledObject<D> {
  style(prompt).green().bold()
}
fn resp_prompt<D>(prompt: D) -> StyledObject<D> {
  style(prompt).red().bold()
}
fn note_prompt<D>(prompt: D) -> StyledObject<D> {
  style(prompt).yellow().dim()
}

fn getline() -> Result<String> {
  let mut line = String::new();
  stdin().read_line(&mut line)?;
  Ok(line)
}

fn main() -> Result<()> {
  env_logger::init();

  // bftpd server listening at localhost:21
  let server_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 21);
  let (mut ftp, welcome) = FtpClient::connect(server_addr)?;
  println!("{}", resp_prompt(welcome.1));

  loop {
    print!("{} ", cmd_prompt("cmd>"));
    stdout().flush().unwrap();
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

fn pick<T: Debug + Clone>(options: &[T]) -> Option<T> {
  println!("{}", note_prompt("no exact match, pick fuzzy match results"));
  if options.is_empty() {
    println!("{}", note_prompt("no fuzzy match candidate"));
    return None;
  }

  for (idx, opt) in options.iter().enumerate() {
    println!("{} {:?}", note_prompt(format!("---choice[{idx}]")), opt)
  }
  print!("{}", note_prompt("pick> "));
  stdout().flush().unwrap();

  let line = getline().ok()?;
  let idx = line.trim().parse::<usize>().ok()?;

  options.get(idx).cloned()
}
