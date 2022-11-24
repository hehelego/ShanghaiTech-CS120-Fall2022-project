use clap::Parser;
use proj2_multiple_access::MacAddr;
use proj3_gateway::IpLayerInternal;
use std::{io::Result, net::Ipv4Addr};

/// IP layer server running on the internal node of Athernet (node1)
#[derive(Parser)]
struct AnetIpCli {
  /// Athernet IP address of gateway node
  gateway_ip: Ipv4Addr,
  /// Athernet MAC address of gateway node
  gateway_mac: u8,
  /// Athernet IP address of internal node
  internal_ip: Ipv4Addr,
  /// Athernet MAC address of internal node
  internal_mac: u8,
}

fn main() -> Result<()> {
  env_logger::init();

  let AnetIpCli {
    gateway_ip,
    gateway_mac,
    internal_ip,
    internal_mac,
  } = AnetIpCli::parse();
  let gateway_mac = MacAddr(gateway_mac);
  let internal_mac = MacAddr(internal_mac);

  let gateway_addr = (gateway_mac, gateway_ip);
  let internal_addr = (internal_mac, internal_ip);
  let mut ip_layer = IpLayerInternal::new(internal_addr, gateway_addr)?;
  ip_layer.run();

  Ok(())
}
