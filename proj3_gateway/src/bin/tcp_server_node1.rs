use clap::Parser;
use std::{
  fs::File,
  io::Write,
  net::{Ipv4Addr, SocketAddrV4},
};

use proj3_gateway::{TcpListener, TcpStream};

#[derive(Parser)]
struct Cli {
  /// host port
  port: u16,
}
fn main() {
  const SELF_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
  let port = Cli::parse().port;
  let self_addr = SocketAddrV4::new(SELF_ADDR, port);
  let tcp_listener = TcpListener::bind(self_addr).unwrap();
  println!("Server start on port {}", port);
  match tcp_listener.accept() {
    Ok((socket, addr)) => accept_file(socket, addr),
    Err(_) => {
      println!("error occured when open connection with the peer")
    }
  }
}

fn accept_file(tcp_stream: TcpStream, addr: SocketAddrV4) {
  let mut output_file = File::create("OUTPUT.txt").unwrap();
  let mut data = [0; 1024];
  while let (size, true) = tcp_stream.read_timeout(&mut data, None) {
    println!("Receive {} bytes from {}", size, addr);
    println!("payloads:\n{}", String::from_utf8_lossy(&data[..size]));
    output_file.write_all(&data[..size]).unwrap();
  }
}
