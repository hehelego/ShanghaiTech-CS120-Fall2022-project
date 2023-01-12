use std::io::{ErrorKind, Read, Result, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::thread::spawn;

const BUF_SIZE: usize = 1024;
const LISTEN_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
const LISTEN_PORT: u16 = 9999;

fn main() -> Result<()> {
    let addr = SocketAddrV4::new(LISTEN_IP, LISTEN_PORT);
    let listener = TcpListener::bind(addr)?;
    println!("socks5 proxy server listening on {:?}", addr);

    for stream in listener.incoming() {
        let stream = stream?;
        let addr = stream.peer_addr()?;
        println!("[{:?}] start", addr);
        spawn(move || {
            let _ = handle_connection(stream, addr);
            println!("[{:?}] end", addr);
        });
    }

    Ok(())
}
fn copy(mut r: impl Read, mut w: impl Write) -> usize {
    let mut m = 0;
    let mut buf = [0; BUF_SIZE];
    loop {
        match dbg!(r.read(&mut buf)) {
            Ok(n) if n > 0 => {
                if w.write_all(&buf[..n]).is_ok() {
                    m += n;
                } else {
                    break;
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            _ => break,
        }
    }

    m
}

fn handle_connection(mut local_stream: TcpStream, local_addr: SocketAddr) -> Result<()> {
    let mut buf = [0; BUF_SIZE];

    // first time interaction: versioning & authentication
    let n = local_stream.read(&mut buf)?;
    assert!(n >= 3); // socks5
    assert_eq!(buf[0], 5); // socks5; no auth
    local_stream.write(&[5, 0])?;

    // second time interaction: connect to host
    let n = local_stream.read(&mut buf)?;
    assert_eq!(n, 10);
    assert_eq!(&buf[0..4], &[5, 1, 0, 1]); // socks5; connect to host; reserved 0; ipv4 addr
    let ip = Ipv4Addr::new(buf[4], buf[5], buf[6], buf[7]);
    let port = (buf[8] as u16) << 8 | (buf[9] as u16);
    let remote_addr = SocketAddrV4::new(ip, port);
    println!("[{:?}] connect to {:?}", local_addr, remote_addr);

    let remote_stream = match TcpStream::connect(remote_addr) {
        Ok(stream) => {
            println!(
                "[{:?}] connection established on port {:?}",
                local_addr,
                stream.local_addr().unwrap().port()
            );
            buf[1] = 0;
            buf[4..10].fill(0);
            // socks5; succeeded; reserved 0; ipv4 addr; bind ip; bind port
            local_stream.write(&buf[..n])?;

            stream
        }
        Err(e) => {
            println!("[{:?}] connection failed {:?}", local_addr, e);
            buf[1] = 1;
            buf[4..10].fill(0);
            // socks5; general error; reserved 0; ipv4 addr; bind ip; bind port
            local_stream.write(&buf[..n])?;

            return Err(e);
        }
    };

    // forward: local -> remote
    let fwd_l2r = {
        let local_stream = local_stream.try_clone()?;
        let remote_stream = remote_stream.try_clone()?;
        spawn(move || {
            copy(&local_stream, &remote_stream);
            let _ = remote_stream.shutdown(Shutdown::Write);
            println!("[{:?}] local -> remote, end", local_addr);
        })
    };
    // forward: remote -> local
    let fwd_r2l = spawn(move || {
        copy(&remote_stream, &local_stream);
        let _ = local_stream.shutdown(Shutdown::Write);
        println!("[{:?}] remote -> local, end", local_addr);
    });

    println!("[{:?}] forwarding data", local_addr);
    let _ = fwd_l2r.join().unwrap();
    let _ = fwd_r2l.join().unwrap();

    Ok(())
}
