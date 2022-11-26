use clap::Parser;
use proj3_gateway::UdpSocket;
use std::{
  fs::File,
  io::{BufRead, BufReader},
  net::{Ipv4Addr, SocketAddrV4},
};

#[derive(Parser)]
struct Cli {
  src_port: u16,
  dest_addr: SocketAddrV4,
}

fn main() {
  let Cli { src_port, dest_addr } = Cli::parse();
  // Initialize socket.
  let self_addr: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 2), src_port);
  let upd_socket = UdpSocket::bind(self_addr).unwrap();

  // Read data
  let data_lines = BufReader::new(File::open("INPUT.txt").unwrap()).lines();
  let mut lines_count = 0;

  println!("Starting send data to {dest_addr}");
  for line in data_lines.flatten() {
    let data = line.as_bytes();
    upd_socket.send_to(data, dest_addr).unwrap();
    println!("Send {} bytes", data.len());
    lines_count += 1;
  }
  // Send two blank line to end the transmission
  upd_socket.send_to("".as_bytes(), dest_addr).unwrap();
  upd_socket.send_to("".as_bytes(), dest_addr).unwrap();
  println!("Send finished. {lines_count} lines in total");
}
