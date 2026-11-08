//! Loopback CONNECT proxy for application WebViews. DNS is resolved here and the
//! connection is made to the checked address, so DNS rebinding cannot bypass the
//! local/private address rule.

use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    sync::{atomic::{AtomicUsize, Ordering}, OnceLock},
    time::Duration,
};

const MAX_CONNECTIONS: usize = 32;
static PORT: OnceLock<Result<u16, String>> = OnceLock::new();
static ACTIVE: AtomicUsize = AtomicUsize::new(0);

pub fn port() -> Result<u16, String> {
    PORT.get_or_init(|| {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .map_err(|e| format!("应用网络代理启动失败: {e}"))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        std::thread::Builder::new().name("app-runtime-proxy".into())
            .spawn(move || listen(listener))
            .map_err(|e| format!("应用网络代理线程启动失败: {e}"))?;
        Ok(port)
    }).clone()
}

fn listen(listener: TcpListener) {
    for incoming in listener.incoming() {
        let Ok(mut client) = incoming else { break };
        if ACTIVE.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
            ACTIVE.fetch_sub(1, Ordering::SeqCst);
            let _ = client.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        if std::thread::Builder::new().name("app-runtime-tunnel".into()).spawn(move || {
            let _ = handle(&mut client);
            ACTIVE.fetch_sub(1, Ordering::SeqCst);
        }).is_err() {
            ACTIVE.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

fn handle(client: &mut TcpStream) -> std::io::Result<()> {
    client.set_read_timeout(Some(Duration::from_secs(8)))?;
    let mut header = Vec::with_capacity(256);
    let mut byte = [0u8; 1];
    while header.len() < 4096 && !header.ends_with(b"\r\n\r\n") {
        client.read_exact(&mut byte)?;
        header.push(byte[0]);
    }
    let target = std::str::from_utf8(&header).ok()
        .and_then(|text| text.split("\r\n").next())
        .and_then(parse_connect);
    let Some((host, port)) = target else { return reject(client, 400, "Bad Request") };
    let addresses: Vec<SocketAddr> = if let Ok(ip) = host.parse::<IpAddr>() {
        vec![SocketAddr::new(ip, port)]
    } else {
        (format!("{host}."), port).to_socket_addrs()?.collect()
    };
    if addresses.is_empty() || addresses.iter().any(|address| !public_address(address.ip())) {
        return reject(client, 403, "Forbidden");
    }
    let mut upstream = match addresses.into_iter().find_map(|address| {
        TcpStream::connect_timeout(&address, Duration::from_secs(3)).ok()
    }) {
        Some(stream) => stream,
        None => return reject(client, 502, "Bad Gateway"),
    };
    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
    let mut hello = [0u8; 5];
    client.read_exact(&mut hello)?;
    if hello[0] != 0x16 || hello[1] != 0x03 {
        return Ok(());
    }
    upstream.write_all(&hello)?;
    client.set_read_timeout(Some(Duration::from_secs(60)))?;
    upstream.set_read_timeout(Some(Duration::from_secs(60)))?;
    let mut client_read = client.try_clone()?;
    let mut upstream_write = upstream.try_clone()?;
    let forward = std::thread::spawn(move || {
        let _ = std::io::copy(&mut client_read, &mut upstream_write);
        let _ = upstream_write.shutdown(Shutdown::Write);
    });
    let _ = std::io::copy(&mut upstream, client);
    let _ = client.shutdown(Shutdown::Write);
    let _ = forward.join();
    Ok(())
}

fn parse_connect(line: &str) -> Option<(String, u16)> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "CONNECT" { return None; }
    let authority = parts.next()?;
    if !matches!(parts.next()?, "HTTP/1.1" | "HTTP/1.0") || parts.next().is_some() { return None; }
    let (host, port) = authority.rsplit_once(':')?;
    let port = port.parse::<u16>().ok().filter(|value| *value != 0)?;
    let host = host.trim_start_matches('[').trim_end_matches(']').to_ascii_lowercase();
    if host.is_empty() || host.len() > 253 { return None; }
    if host.parse::<IpAddr>().is_err() &&
        (host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") ||
         host.ends_with(".internal") || host.ends_with(".home.arpa") || host.ends_with(".lan") ||
         !host.contains('.') || host.split('.').any(|label| label.is_empty() || label.starts_with('-') ||
             label.ends_with('-') || !label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')))
    { return None; }
    Some((host, port))
}

fn public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !ip.is_private() && !ip.is_loopback() && !ip.is_link_local() &&
                !ip.is_multicast() && !ip.is_broadcast() && !ip.is_unspecified() &&
                a != 0 && a < 224 && !(a == 100 && (64..=127).contains(&b)) &&
                !(a == 198 && (b == 18 || b == 19)) &&
                !(a == 192 && b == 0 && c == 0) &&
                !(a == 192 && b == 0 && c == 2) &&
                !(a == 198 && b == 51 && c == 100) &&
                !(a == 203 && b == 0 && c == 113)
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ip.to_ipv4_mapped() { return public_address(IpAddr::V4(v4)); }
            !ip.is_loopback() && !ip.is_unspecified() && !ip.is_unique_local() &&
                !ip.is_unicast_link_local() && !ip.is_multicast()
        }
    }
}

fn reject(client: &mut TcpStream, code: u16, reason: &str) -> std::io::Result<()> {
    client.write_all(format!("HTTP/1.1 {code} {reason}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_targets_are_rejected() {
        for address in ["127.0.0.1", "10.1.2.3", "172.16.0.1", "192.168.0.1", "169.254.1.1", "100.64.0.1", "::1", "fc00::1", "fe80::1", "::ffff:127.0.0.1"] {
            assert!(!public_address(address.parse().unwrap()), "{address}");
        }
        assert!(public_address("8.8.8.8".parse().unwrap()));
        assert!(public_address("2606:4700:4700::1111".parse().unwrap()));
        assert!(parse_connect("CONNECT localhost:443 HTTP/1.1").is_none());
        assert!(parse_connect("CONNECT example.com:443 HTTP/1.1").is_some());
    }

    #[test]
    fn proxy_refuses_a_loopback_tunnel() {
        let mut client = TcpStream::connect((Ipv4Addr::LOCALHOST, port().unwrap())).unwrap();
        client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        client.write_all(b"CONNECT 127.0.0.1:443 HTTP/1.1\r\nHost: 127.0.0.1:443\r\n\r\n").unwrap();
        let mut response = [0u8; 128];
        let count = client.read(&mut response).unwrap();
        assert!(String::from_utf8_lossy(&response[..count]).starts_with("HTTP/1.1 403"));
    }
}
