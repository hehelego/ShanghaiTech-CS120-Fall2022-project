use super::ipc::{recv_packet, send_packet, IPC_TIMEOUT};
use proj2_multiple_access::{MacAddr, MacLayer};
use socket2::{Domain, SockAddr, Socket, Type};

/// A unix-domain socket server which provides IP layer service.
/// It allow sending/receiving network layer packet to peers via MAC layer.
///
/// - hold an unique mac layer object
/// - communicate with other processes to perform
///   - IP packet send/receive
///   - socket bind/unbind
///   via unix domain socket IPC
pub struct IpProvider {
  mac: MacLayer,
  sock: Socket,
}

impl IpProvider {
  /// The UNIX domain socket on which the IP layer provider is bind
  pub const SOCK_PATH: &str = "/tmp/athernet_ip_server.sock";

  pub fn sock_path() -> SockAddr {
    SockAddr::unix(Self::SOCK_PATH).unwrap()
  }

  /// create a IP layer [`IpProvider`], prepare to run the server.
  /// **NOTE** On every Athernet node, only one Provider instance should exists.
  pub fn new(mac_addr: MacAddr) -> std::io::Result<Self> {
    // unix domain socket creation
    let _ = std::fs::remove_file(Self::SOCK_PATH);
    let sock = Socket::new(Domain::UNIX, Type::DGRAM, None)?;
    sock.set_read_timeout(Some(IPC_TIMEOUT))?;
    sock.set_write_timeout(Some(IPC_TIMEOUT))?;
    sock.bind(&Self::sock_path())?;
    // mac layer creation
    let mac = MacLayer::new_with_default_phy(mac_addr);

    Ok(Self { mac, sock })
  }

  /// run forever
  pub fn serve(self) {
    todo!()
  }
}
