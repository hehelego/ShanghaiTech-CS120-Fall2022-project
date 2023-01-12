/// Wrap of Athernet TCP socket
mod asock;

/// SOCKS5 protocol proxy server that
/// forwards normal network traffic through Athernet.
mod proxy_server;

/// FTP client built on Athernet TCP transport layer.
mod ftp_client;

#[cfg(test)]
mod tests;
