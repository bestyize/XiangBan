//! Minimal, silent synchronous Claude Code hook forwarder.
#![forbid(unsafe_code)]
use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream},
    time::{Duration, Instant},
};
use xiaoban_agent_hooks::endpoint;
fn main() {
    let _ = run();
}
fn run() -> Option<()> {
    let mut args = std::env::args().skip(1);
    let event = args.next()?;
    if event.is_empty()
        || !event
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return None;
    }
    let mut wait = 125u64;
    while let Some(arg) = args.next() {
        if arg == "--wait" {
            wait = args.next()?.parse().ok()?;
        }
    }
    let deadline =
        Instant::now().checked_add(Duration::from_secs(if event == "PermissionRequest" {
            wait
        } else {
            2
        }))?;
    let ep = endpoint::read().ok()?;
    if ep.token.is_empty() || !ep.token.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut input = Vec::new();
        let result = io::stdin()
            .take(1024 * 1024 + 1)
            .read_to_end(&mut input)
            .map(|_| input);
        let _ = tx.send(result);
    });
    let input = rx
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .ok()?
        .ok()?;
    if input.len() > 1024 * 1024 {
        return None;
    }
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, ep.port));
    let mut stream = TcpStream::connect_timeout(
        &addr,
        Duration::from_millis(500).min(deadline.saturating_duration_since(Instant::now())),
    )
    .ok()?;
    stream
        .set_write_timeout(Some(deadline.saturating_duration_since(Instant::now())))
        .ok()?;
    let header = format!("POST /h/{}/{event} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", ep.token, ep.port, input.len());
    stream.write_all(header.as_bytes()).ok()?;
    stream.write_all(&input).ok()?;
    let mut response = Vec::new();
    let mut buf = [0; 4096];
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        stream.set_read_timeout(Some(remaining)).ok()?;
        let n = stream.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        response.extend_from_slice(&buf[..n]);
        if response.len() > 1024 * 1024 + 8192 {
            return None;
        }
    }
    let split = response.windows(4).position(|w| w == b"\r\n\r\n")?;
    if !response.starts_with(b"HTTP/1.1 200 ") {
        return None;
    }
    io::stdout().write_all(&response[split + 4..]).ok()?;
    Some(())
}
