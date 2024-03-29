use proj3_gateway::TcpStream;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{Shutdown, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

const ASOCK_TIMEOUT: Duration = Duration::from_secs(2);
const ASOCK_FLUSH_TIME: Duration = Duration::from_millis(50);

/// Wrapper for Athernet TCP socket, provide Reader/Writer implementation
#[derive(Clone)]
pub struct ASocket(Arc<TcpStream>);

impl ASocket {
  pub fn bind(addr: SocketAddrV4) -> Result<ASocket> {
    TcpStream::bind(addr)
      .map(|sock| ASocket(Arc::new(sock)))
      .map_err(|_| Error::new(ErrorKind::AddrInUse, "address in use, bind failed".to_string()))
  }

  pub fn connect(&mut self, dest: SocketAddrV4) -> Result<()> {
    Arc::get_mut(&mut self.0)
      .expect("Athernet Tcp Socket borrow mut")
      .connect(dest)
      .map_err(|_| {
        Error::new(
          ErrorKind::ConnectionRefused,
          "cannot connect to remote host".to_string(),
        )
      })
  }

  pub fn shutdown(&mut self, how: Shutdown) -> Result<()> {
    match how {
      Shutdown::Read => self.0.shutdown_read(),
      Shutdown::Write => self.0.shutdown_write(),
      Shutdown::Both => self.0.shutdown_both(),
    }
    .map_err(|_| Error::new(ErrorKind::Other, "connection shutdown error".to_string()))
  }
}

impl Read for ASocket {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    let (n, fin) = self.0.read_timeout(buf, Some(ASOCK_TIMEOUT));
    if n > 0 {
      Ok(n)
    } else if !fin {
      Err(Error::new(ErrorKind::Interrupted, "wait for incoming data".to_string()))
    } else {
      Ok(0)
    }
  }
}

impl Write for ASocket {
  fn write(&mut self, buf: &[u8]) -> Result<usize> {
    self
      .0
      .write_timeout(buf, None)
      .map_err(|_| Error::new(ErrorKind::Other, "unknown error".to_string()))
  }

  fn flush(&mut self) -> Result<()> {
    std::thread::sleep(ASOCK_FLUSH_TIME);
    Ok(())
  }
}
