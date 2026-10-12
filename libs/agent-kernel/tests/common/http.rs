//! A tiny fake HTTP server for provider tests: canned responses, requests recorded verbatim.

use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const SSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n";

pub fn read_request(stream: &mut TcpStream) -> String {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while !buf.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte).unwrap_or(0) == 0 {
            break;
        }
        buf.push(byte[0]);
    }
    let head = String::from_utf8_lossy(&buf).to_string();
    let length = head.to_lowercase().lines().find_map(|l| l.strip_prefix("content-length: ")?.trim().parse().ok()).unwrap_or(0);
    let mut body = vec![0u8; length];
    let _ = stream.read_exact(&mut body);
    head + &String::from_utf8_lossy(&body)
}

/// Answers each incoming connection with the next canned response, then closes it.
/// The URL ends in `/v1`; the join handle yields every request received, headers and body.
pub fn serve(responses: Vec<String>) -> (String, JoinHandle<Vec<String>>) {
    serve_holding(responses, Duration::ZERO)
}

/// Like `serve`, but keeps each connection open for `hold` after writing, like a stalled stream.
pub fn serve_holding(responses: Vec<String>, hold: Duration) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut seen = Vec::new();
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            seen.push(read_request(&mut stream));
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            thread::sleep(hold);
        }
        seen
    });
    (url, handle)
}

/// An SSE response whose events carry only `data:` lines.
pub fn sse(chunks: &[&str]) -> String {
    let events: String = chunks.iter().map(|c| format!("data: {c}\n\n")).collect();
    format!("{SSE}{events}")
}

/// An SSE response with `event:` and `data:` lines, as the Messages and Responses APIs send them.
pub fn sse_events(events: &[(&str, &str)]) -> String {
    let body: String = events.iter().map(|(name, data)| format!("event: {name}\ndata: {data}\n\n")).collect();
    format!("{SSE}{body}")
}

pub fn status(line: &str, body: &str) -> String {
    format!("HTTP/1.1 {line}\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}", body.len())
}

/// Values of every header called `name` (case-insensitive) in a recorded request.
pub fn header_values(request: &str, name: &str) -> Vec<String> {
    let head = request.split("\r\n\r\n").next().unwrap_or("");
    head.lines()
        .skip(1)
        .filter_map(|l| l.split_once(':'))
        .filter(|(k, _)| k.trim().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim().to_string())
        .collect()
}

pub fn request_path(request: &str) -> String {
    request.split_whitespace().nth(1).unwrap_or("").to_string()
}

pub fn body_json(request: &str) -> Value {
    serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap_or("")).expect("request body is JSON")
}
