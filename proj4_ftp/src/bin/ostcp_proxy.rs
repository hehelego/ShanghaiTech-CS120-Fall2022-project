use std::io::{copy, Read, Result, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::thread::spawn;

const BUF_SIZE: usize = 1024;
const LISTEN_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
const LISTEN_PORT: u16 = 9999;

fn main() -> Result<()> {
  env_logger::init();

  let addr = SocketAddrV4::new(LISTEN_IP, LISTEN_PORT);
  let listener = TcpListener::bind(addr)?;
  log::debug!("socks5 proxy server listening on {:?}", addr);

  for stream in listener.incoming() {
    let stream = stream?;
    let addr = stream.peer_addr()?;
    log::debug!("start servering {:?}", addr);
    spawn(move || {
      let _ = handle_connection(stream, addr);
      log::debug!("end servering {:?}", addr);
    });
  }

  Ok(())
}

fn handle_connection(mut local_stream: TcpStream, local_addr: SocketAddr) -> Result<()> {
  let mut buf = [0; BUF_SIZE];

  // first time interaction: versioning & authentication
  local_stream.read_exact(&mut buf[..3])?;
  assert_eq!(buf[0], 5); // socks5; no auth
  local_stream.write_all(&[5, 0])?;

  // second time interaction: connect to host. get port number
  local_stream.read_exact(&mut buf[..10])?;
  assert_eq!(&buf[0..4], &[5, 1, 0, 1]); // socks5; connect to host; reserved 0; ipv4 addr
  let ip = Ipv4Addr::new(buf[4], buf[5], buf[6], buf[7]);
  let port = (buf[8] as u16) << 8 | (buf[9] as u16);
  let remote_addr = SocketAddrV4::new(ip, port);
  log::debug!("{:?} connect to {:?}", local_addr, remote_addr);

  let mut remote_stream = match TcpStream::connect(remote_addr) {
    Ok(stream) => {
      log::debug!(
        "{:?} <-> {:?} connection established on port {:?}",
        local_addr,
        remote_addr,
        stream.local_addr().map(|x| x.port())
      );
      buf[1] = 0;
      buf[4..10].fill(0);
      // socks5; succeeded; reserved 0; ipv4 addr; bind ip; bind port
      local_stream.write_all(&buf[..10])?;

      stream
    }
    Err(e) => {
      log::debug!("{:?} <-> {:?} connect failed: {:?}", local_addr, remote_addr, e);
      buf[1] = 1;
      buf[4..10].fill(0);
      // socks5; general error; reserved 0; ipv4 addr; bind ip; bind port
      local_stream.write_all(&buf[..10])?;

      return Err(e);
    }
  };

  // forward: local -> remote
  let fwd_l2r = {
    let mut local_stream = local_stream.try_clone()?;
    let mut remote_stream = remote_stream.try_clone()?;
    spawn(move || {
      let fwd_bytes = copy(&mut local_stream, &mut remote_stream);
      let _ = remote_stream.shutdown(Shutdown::Write);
      log::debug!(
        "local({:?}) -> remote({:?}), end. {:?} bytes forwarded",
        local_addr,
        remote_addr,
        fwd_bytes
      );
    })
  };
  // forward: remote -> local
  let fwd_r2l = spawn(move || {
    let fwd_bytes = copy(&mut remote_stream, &mut local_stream);
    let _ = local_stream.shutdown(Shutdown::Write);
    log::debug!(
      "remote({:?}) -> local({:?}), end. {:?} bytes forwarded",
      remote_addr,
      local_addr,
      fwd_bytes
    );
  });

  log::debug!(
    "start data forwarding for local({:?}) <-> remote({:?})",
    local_addr,
    remote_addr
  );
  fwd_l2r.join().unwrap();
  fwd_r2l.join().unwrap();

  Ok(())
}
