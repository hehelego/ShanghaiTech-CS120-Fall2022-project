use clap::Parser;
use std::{fs::File, io::Write, net::UdpSocket};

#[derive(Parser)]
struct Cli {
  /// host port
  port: u16,
}
fn main() {
  const SELF_ADDR: &str = "0.0.0.0";
  const DATA_SIZE: usize = 40;

  let port = Cli::parse().port;
  let udp_socket = UdpSocket::bind((SELF_ADDR, port)).unwrap();
  let mut output_file = File::create("OUTPUT.txt").unwrap();
  let mut buf = [0u8; DATA_SIZE];
  let mut blank_line: bool = false;
  loop {
    if let Ok((size, src_dest)) = udp_socket.recv_from(&mut buf) {
      println!("Receive {} bytes from {}", size, src_dest);
      if size == 0 {
        if blank_line {
          println!("Finish receiving.");
          break;
        }
        blank_line = true;
      } else {
        if blank_line {
          blank_line = false;
          writeln!(output_file).unwrap();
        }
        writeln!(output_file, "{}", String::from_utf8(buf[..size].to_vec()).unwrap()).unwrap();
      }
    }
  }
}
