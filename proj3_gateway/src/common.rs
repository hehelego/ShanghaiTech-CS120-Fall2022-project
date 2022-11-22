use std::time::Duration;

/// timeout value for IPC unix domain socket send/recv
pub const IPC_TIMEOUT: Duration = Duration::from_millis(10);

/// timeout value for RAW IP socket send/recv
pub const RAWSOCK_TIMEOUT: Duration = Duration::from_millis(10);

/// timeout value for MAC layer send/recv
pub const MAC_TIMEOUT: Duration = Duration::from_millis(10);

/// The UNIX domain socket on which the IP layer provider is bind
pub const AIP_SOCK: &str = "/tmp/athernet_ip_server.sock";
