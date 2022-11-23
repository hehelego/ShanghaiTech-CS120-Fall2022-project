use proj2_multiple_access::MacAddr;
use proj3_gateway::IpLayerGateway;
use std::net::Ipv4Addr;

fn main() {
  const SELF_MAC: MacAddr = MacAddr(2);
  const SELF_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 1);
  const PEER_MAC: MacAddr = MacAddr(1);
  const PEER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 2);
  let mut ip_layer = IpLayerGateway::new((SELF_MAC, SELF_IP), (PEER_MAC, PEER_IP)).unwrap();
  ip_layer.run();
}
