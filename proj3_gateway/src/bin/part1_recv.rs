use clap::Parser;
use std::net::UdpSocket;

#[derive(Parser)]
struct Cli {
  /// host port
  port: u16,
}

fn main() -> std::io::Result<()> {
  env_logger::init();

  const SELF_ADDR: &str = "0.0.0.0";
  const DATA_SIZE: usize = 20;
  const DATA_NUM: usize = 10;

  let Cli { port } = Cli::parse();

  let upd_socket = UdpSocket::bind((SELF_ADDR, port))?;
  println!("Udp socket bind on {SELF_ADDR}:{port}");

  let mut buffer = [0; DATA_SIZE];
  let mut data_recv_count = 0;
  while data_recv_count < DATA_NUM {
    let (bytes, src) = upd_socket.recv_from(&mut buffer)?;
    data_recv_count += 1;
    println!("[{data_recv_count}] Receive {bytes} bytes data from {src}");
  }

  println!("Finish receiveing...exit.");
  Ok(())
}
