/// IP packet, and TCP/UDP/ICMP packet inside the payload.
mod packet;

/// Athernet IP layer service provider and accessor
mod aip_layer;
pub use aip_layer::{IpAccessor, IpLayerGateway, IpLayerInternal};

/// Transport layer socket API built on Athernet
mod socket;
pub use socket::{ASockProtocol, IcmpSocket, TcpSocket, UdpSocket};

/// Define common constant values: timeout length, maximum packet size ...
mod common;

#[cfg(test)]
mod tests;
