use proj2_multiple_access::MacAddr;
use proj3_gateway::IpLayerGateway;
use std::{io::Result, net::Ipv4Addr};

use clap::Parser;

/// NAT server running on the gateway of Athernet (node2)
#[derive(Parser)]
struct NatCli {
  /// Athernet IP address of gateway node
  gateway_ip: Ipv4Addr,
  /// Athernet MAC address of gateway node
  gateway_mac: u8,
  /// Athernet IP address of internal node
  internal_ip: Ipv4Addr,
  /// Athernet MAC address of internal node
  internal_mac: u8,

  /// Internet IP address of the gateway/NAT
  nat_ip: Ipv4Addr,
}

fn main() -> Result<()> {
  env_logger::init();

  let NatCli {
    gateway_ip,
    gateway_mac,
    internal_ip,
    internal_mac,
    nat_ip,
  } = NatCli::parse();
  let gateway_mac = MacAddr(gateway_mac);
  let internal_mac = MacAddr(internal_mac);

  let gateway_addr = (gateway_mac, gateway_ip);
  let internal_addr = (internal_mac, internal_ip);
  let mut ip_layer = IpLayerGateway::new(gateway_addr, internal_addr, nat_ip)?;
  ip_layer.run();

  Ok(())
}
