use clap::Parser;
use std::{
  fs::File,
  io::{BufRead, BufReader},
  net::UdpSocket,
};

#[derive(Parser)]
struct Cli {
  /// The host port
  src_port: u16,
  /// The ip address.
  dest_addr: String,
  /// The port
  dest_port: u16,
}

fn main() -> std::io::Result<()> {
  // Constants
  const UDP_ADDR: &str = "0.0.0.0";

  // Bind the socket
  let Cli {
    src_port,
    dest_addr,
    dest_port,
  } = Cli::parse();
  let udp_socket = UdpSocket::bind((UDP_ADDR, src_port))?;
  println!("Udp bind on: {UDP_ADDR}:{src_port}");

  // Read data
  let data_lines = BufReader::new(File::open("INPUT.txt").unwrap()).lines();
  let mut lines_count = 0;
  println!("Starting send data to {dest_addr}:{dest_port}");
  let dest_addr = (dest_addr.as_str(), dest_port);
  for line in data_lines.flatten() {
    let data = line.as_bytes();
    udp_socket.send_to(data, dest_addr).unwrap();
    println!("Send {} bytes", data.len());
    lines_count += 1;
  }
  // Send two blank line to end the transmission
  udp_socket.send_to("".as_bytes(), dest_addr).unwrap();
  udp_socket.send_to("".as_bytes(), dest_addr).unwrap();
  println!("Send finished. {lines_count} lines in total");
  Ok(())
}
