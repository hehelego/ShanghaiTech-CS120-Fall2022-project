use clap::Parser;
use std::{
  fs::File,
  io::{Read, Write},
  net::{SocketAddr, TcpListener, TcpStream},
};

#[derive(Parser)]
struct Cli {
  /// host port
  port: u16,
}
fn main() {
  const SELF_ADDR: &str = "0.0.0.0";

  let port = Cli::parse().port;
  let tcp_listener = TcpListener::bind((SELF_ADDR, port)).unwrap();
  println!("Server start on port {}", port);
  match tcp_listener.accept() {
    Ok((socket, addr)) => accept_file(socket, addr),
    Err(_) => {
      println!("error occured when open connection with the peer")
    }
  }
}

fn accept_file(mut tcp_stream: TcpStream, addr: SocketAddr) {
  let mut output_file = File::create("OUTPUT.txt").unwrap();
  let mut data = Vec::new();
  tcp_stream.read_to_end(&mut data).unwrap();
  println!("Receive {} bytes from {}", data.len(), addr);
  println!("payloads:\n{}", String::from_utf8(data.clone()).unwrap());
  writeln!(output_file, "{}", String::from_utf8(data).unwrap()).unwrap();
}
