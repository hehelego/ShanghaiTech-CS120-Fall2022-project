use clap::Parser;
use proj3_gateway::TcpStream;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
struct Cli {
  src_port: u16,
  dest_addr: SocketAddrV4,
}

fn main() {
  env_logger::init();
  let Cli { src_port, dest_addr } = Cli::parse();
  // Initialize socket.
  let self_addr: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 2), src_port);
  let mut tcp_stream = TcpStream::bind(self_addr).unwrap();
  tcp_stream.connect(dest_addr).unwrap();

  // Read data
  let data_lines = std::fs::read("INPUT.txt").unwrap();
  println!("Starting send data to {dest_addr}");
  match tcp_stream.write_timeout(&data_lines, None) {
    Ok(size) => println!("Send {} bytes", size),
    Err(_) => println!("Send data error"),
  }
  match tcp_stream.shutdown() {
    Ok(_) => println!("Send data finished, tcp stream shutdown"),
    Err(_) => println!("Unable to shutdown the tcp stream"),
  }
}
