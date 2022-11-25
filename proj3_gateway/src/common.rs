use socket2::SockAddr;
use std::time::Duration;

/// Time to wait before unix domain socket IPC retry
pub const IPC_RETRY_WAIT: Duration = Duration::from_millis(10);

/// Maximum packet size for IPC packet send over unix domain socket
pub const IPC_PACK_SIZE: usize = 4096;

/// Maximum packet size for RAW socket IPv4 packet
pub const RAWIP_PACK_SIZE: usize = 4096;

/// timeout value for IPC unix domain socket send/recv
pub const IPC_TIMEOUT: Duration = Duration::from_millis(10);

/// timeout value for RAW IP socket send/recv
pub const RAWSOCK_TIMEOUT: Duration = Duration::from_millis(10);

/// NAT will forward external ICMP ping request into LAN
/// if the payload has a specific substring
pub const NAT_ICMP_BYPASS_PATTERN: &str = "Freiheit";

/// The UNIX domain socket on which the IP layer provider is bind
pub const AIP_SOCK: &str = "/tmp/athernet_ip_server.sock";
pub fn aip_ipc_sockaddr() -> SockAddr {
  SockAddr::unix(AIP_SOCK).unwrap()
}
