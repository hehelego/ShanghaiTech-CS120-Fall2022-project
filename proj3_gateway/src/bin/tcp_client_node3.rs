use clap::Parser;
use std::net::TcpStream;

use std::io::Write;
#[derive(Parser)]
struct Cli {
  /// The ip address.
  dest_addr: String,
  /// The port
  dest_port: u16,
}

fn main() -> std::io::Result<()> {
  // Bind the socket
  let Cli { dest_addr, dest_port } = Cli::parse();
  let mut tcp_stream = TcpStream::connect((dest_addr.clone(), dest_port))?;
  println!(
    "Tcp connect to {dest_addr}:{dest_port}, host port: {}",
    tcp_stream.local_addr().unwrap()
  );

  // Read data
  let data_lines = std::fs::read("INPUT.txt").unwrap();
  println!("Starting send data to {dest_addr}:{dest_port}");
  tcp_stream.write_all(&data_lines).unwrap();
  println!("Data sent");
  tcp_stream.shutdown(std::net::Shutdown::Write)?;
  Ok(())
}
