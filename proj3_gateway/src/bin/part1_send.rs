use clap::Parser;
use std::{
  net::UdpSocket,
  thread,
  time::{Duration, Instant},
};

use rand::{self, RngCore};

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
  env_logger::init();

  // Constants
  const PAYLOAD_BYTES: usize = 20;
  const LOOP_TIME: usize = 10;
  const SEND_INTERVAL: Duration = Duration::from_secs(1);
  const UDP_ADDR: &str = "0.0.0.0";

  // Delay the start for 1 secs
  thread::sleep(Duration::from_secs(1));
  let program_start_time = Instant::now();

  // Bind the socket
  let Cli {
    src_port,
    dest_addr,
    dest_port,
  } = Cli::parse();
  let node3_udp_socket = UdpSocket::bind((UDP_ADDR, src_port))?;
  println!("Udp bind on: {UDP_ADDR}:{src_port}");

  // Initialization
  let mut random_payload = [0; PAYLOAD_BYTES];
  let mut loop_time = 0;

  while loop_time < LOOP_TIME {
    let start_time = Instant::now();
    // Fill the random byte
    rand::thread_rng().fill_bytes(&mut random_payload);
    // Send random data
    node3_udp_socket.send_to(&random_payload, (dest_addr.as_str(), dest_port))?;
    loop_time += 1;
    let end_time = Instant::now();
    // Information for the user.
    println!(
      "[{loop_time}] Send {PAYLOAD_BYTES} bytes to {dest_addr}:{dest_port} at {}s",
      (end_time - program_start_time).as_secs_f32()
    );
    // Sleep for 1s.
    if start_time.elapsed() < SEND_INTERVAL {
      thread::sleep(SEND_INTERVAL - (start_time.elapsed()));
    }
  }
  println!("Finish sending...exit");
  Ok(())
}
