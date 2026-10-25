//! Standard-library TCP fixture shared by kernel and connector integration tests.
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[derive(Clone, Default)]
pub struct Options {
    pub endpoint: Option<String>,
    pub absolute: bool,
    pub reorder: bool,
    pub drop_call: bool,
    pub stall_call: bool,
    pub stall_endpoint: bool,
    pub oversized: bool,
    pub token: Option<String>,
    pub reject_get_once: bool,
    pub reject_post_once: bool,
}
pub struct Server {
    pub base: String,
    pub requests: Arc<Mutex<Vec<(String, Value)>>>,
    pub stream_closed: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
fn event(stream: &mut TcpStream, value: Value) {
    // Multi-line JSON data, comments, retry and CRLF, fragmented across writes.
    let text = serde_json::to_string_pretty(&value).unwrap();
    stream
        .write_all(b": keepalive\r\nretry: 100\r\nevent: message\r\n")
        .unwrap();
    for line in text.lines() {
        write!(stream, "data: {line}\r\n").unwrap();
    }
    stream.write_all(b"\r").unwrap();
    stream.write_all(b"\n").unwrap();
}
impl Server {
    pub fn new(options: Options) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let origin = base.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let quit = stop.clone();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let log = requests.clone();
        let stream_closed = Arc::new(AtomicBool::new(false));
        let disconnected = stream_closed.clone();
        let thread = std::thread::spawn(move || {
            let mut stream: Option<TcpStream> = None;
            let mut calls = Vec::new();
            let mut rejected_get = false;
            let mut rejected_post = false;
            while !quit.load(Ordering::SeqCst) {
                let (mut socket, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if let Some(s) = &stream {
                            let mut b = [0];
                            if matches!(s.peek(&mut b), Ok(0)) {
                                disconnected.store(true, Ordering::SeqCst);
                            }
                        }
                        std::thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(e) => panic!("accept: {e}"),
                };
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                socket
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                let mut first = String::new();
                reader.read_line(&mut first).unwrap();
                let mut headers = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line.trim().is_empty() {
                        break;
                    }
                    if let Some((k, v)) = line.split_once(':') {
                        if k.eq_ignore_ascii_case("content-length") {
                            length = v.trim().parse().unwrap();
                        }
                    }
                    headers.push_str(&line.to_lowercase());
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                let request: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
                log.lock().unwrap().push((first.clone(), request.clone()));
                let get = first.starts_with("GET ");
                if (get && options.reject_get_once && !rejected_get)
                    || (!get && options.reject_post_once && !rejected_post)
                {
                    if get {
                        rejected_get = true;
                    } else {
                        rejected_post = true;
                    }
                    socket.write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                    continue;
                }
                if let Some(token) = &options.token {
                    assert!(
                        headers
                            .contains(&format!("authorization: bearer {}", token.to_lowercase())),
                        "{headers}"
                    );
                }
                if get {
                    assert!(first.starts_with("GET /legacy/sse "));
                    assert!(headers.contains("accept: text/event-stream"));
                    socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n").unwrap();
                    let endpoint = options.endpoint.clone().unwrap_or_else(|| {
                        if options.absolute {
                            format!("{origin}/legacy/messages?session=1")
                        } else {
                            "messages?session=1".into()
                        }
                    });
                    if !options.stall_endpoint {
                        write!(socket, ": comment\nevent: endpoint\ndata: {endpoint}\n\n").unwrap();
                    }
                    socket.set_nonblocking(true).unwrap();
                    stream = Some(socket);
                    continue;
                }
                assert!(
                    first.starts_with("POST /legacy/messages?session=1 "),
                    "{first}"
                );
                assert!(headers.contains("content-type: application/json"));
                socket
                    .write_all(
                        b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .unwrap();
                if request.get("method").is_none() {
                    continue;
                } // replies to server ping
                let method = request["method"].as_str().unwrap();
                let result = match method {
                    "initialize" => {
                        assert_eq!(request["params"]["protocolVersion"], "2024-11-05");
                        json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"legacy","version":"1"}})
                    }
                    "notifications/initialized" => {
                        assert!(request.get("id").is_none());
                        continue;
                    }
                    "tools/list" => {
                        if request["params"]["cursor"] == "second" {
                            json!({"tools":[{"name":"write","inputSchema":{"type":"object"}}]})
                        } else {
                            event(
                                stream.as_mut().unwrap(),
                                json!({"jsonrpc":"2.0","id":"server-ping","method":"ping"}),
                            );
                            event(
                                stream.as_mut().unwrap(),
                                json!({"jsonrpc":"2.0","method":"notifications/progress"}),
                            );
                            event(
                                stream.as_mut().unwrap(),
                                json!({"jsonrpc":"2.0","id":99999,"result":{}}),
                            );
                            json!({"tools":[{"name":"read","inputSchema":{"type":"object"}}],"nextCursor":"second"})
                        }
                    }
                    "tools/call" => {
                        if options.drop_call {
                            stream.take();
                            continue;
                        }
                        if options.stall_call {
                            continue;
                        }
                        if options.oversized {
                            let s = stream.as_mut().unwrap();
                            s.set_nonblocking(false).unwrap();
                            let _ = s.write_all(&vec![b'x'; 8 * 1024 * 1024 + 1]);
                            continue;
                        }
                        json!({"content":[{"type":"text","text":request["params"]["arguments"]["text"].as_str().unwrap_or("ok")}],"structuredContent":{"ok":true}})
                    }
                    _ => panic!("Unexpected request: {request}"),
                };
                let reply = json!({"jsonrpc":"2.0","id":request["id"],"result":result});
                if method == "tools/call" && options.reorder {
                    calls.push(reply);
                    if calls.len() == 2 {
                        for reply in calls.drain(..).rev() {
                            event(stream.as_mut().unwrap(), reply);
                        }
                    }
                } else {
                    event(stream.as_mut().unwrap(), reply);
                }
            }
        });
        Self {
            base,
            requests,
            stream_closed,
            stop,
            thread: Some(thread),
        }
    }
    pub fn url(&self) -> String {
        format!("{}/legacy/sse", self.base)
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.thread.take().unwrap().join().unwrap();
    }
}
