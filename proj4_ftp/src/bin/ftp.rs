use std::io::Result;
use std::net::SocketAddrV4;

use proj4_ftp::{
  cli_util::{cmd_prompt, flush_stdout, getline, pick, resp_prompt},
  ftp_client::{FtpClient, FtpCmd},
};

use clap::Parser;

#[derive(Parser)]
struct AnetFtp {
  /// source address for TCP socket bind
  local_addr: SocketAddrV4,
  /// address of FTP server
  server_addr: SocketAddrV4,
}

fn main() -> Result<()> {
  env_logger::init();
  let AnetFtp {
    local_addr,
    server_addr,
  } = AnetFtp::parse();

  let (mut ftp, welcome) = FtpClient::connect(local_addr, server_addr)?;
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
