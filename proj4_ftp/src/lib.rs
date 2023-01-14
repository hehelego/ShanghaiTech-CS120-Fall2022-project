/// Wrap of Athernet TCP socket
mod asock;
pub use asock::ASocket as AnetTcpSocket;

/// SOCKS5 protocol proxy server that
/// forwards normal network traffic through Athernet.
pub mod proxy_server;

/// FTP client built on Athernet TCP transport layer.
pub mod ftp_client;

/// Utilities for CLI, console text style
pub mod cli_util;


#[cfg(test)]
mod tests;
