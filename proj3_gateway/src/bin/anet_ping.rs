use clap::{Parser, Subcommand};
use proj3_gateway::IcmpSocket;
use std::io::{BufRead, BufReader, ErrorKind, Result};
use std::net::Ipv4Addr;
use std::process;
use std::time::Instant;

const DEFAULT_PING_MSG: &str = "across the great wall we can reach the world";

/// Send ICMP ping from Athernet node
#[derive(Parser)]
struct AnetPing {
  #[command(subcommand)]
  command: Ping,

  #[arg(long, default_value_t = 10)]
  /// the maximum number of ping-pong rounds
  rounds: u32,

  #[arg(long, default_value_t = String::from(DEFAULT_PING_MSG))]
  /// ICMP echo request payload string
  payload: String,
}

#[derive(Subcommand, Clone)]
enum Ping {
  /// Ping to a host specified by IPv4 address
  Direct {
    /// Destination IPv4 address
    ip: Ipv4Addr,
  },
  /// Ping to a host specified by host name
  Dns {
    /// Destination host name
    host: String,
  },
}

fn dns_resolve(host: String) -> Result<Ipv4Addr> {
  let stdout = process::Command::new("dig").arg("+short").arg(host).output()?.stdout;
  let resolved = BufReader::new(&stdout[..]).lines().filter_map(|x| x.ok());
  let parsed = resolved.filter_map(|x| x.parse().ok());
  parsed.last().ok_or_else(|| ErrorKind::InvalidInput.into())
}

fn main() -> Result<()> {
  env_logger::init();

  let AnetPing {
    command: dest,
    rounds,
    payload,
  } = AnetPing::parse();

  let dest = match dest {
    Ping::Direct { ip } => ip,
    Ping::Dns { host } => dns_resolve(host).expect("DNS resolve failed"),
  };
  println!("PING {:?} with: {}", dest, payload);

  let sock = IcmpSocket::bind(Ipv4Addr::new(192, 168, 1, 2)).expect("cannot create ICMP socket");

  let id = rand::random();
  let mut total_time = 0;
  for seq in (0..rounds).map(|x| x as u16) {
    let start = Instant::now();
    sock.send_ping(id, seq, payload.as_bytes(), dest).unwrap();
    let reply_payload = sock.recv_pong(id, seq, dest).unwrap();
    let rtt = start.elapsed().as_millis() as u32;
    total_time += rtt;
    println!(
      "REPLY(seq={}) from {:?} RTT={}ms with: {:?}",
      seq,
      dest,
      rtt,
      String::from_utf8(reply_payload),
    );
  }
  println!("average RTT of {} packets is {}", rounds, total_time / rounds);

  Ok(())
}
