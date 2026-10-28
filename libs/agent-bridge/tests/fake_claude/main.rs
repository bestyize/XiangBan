use serde_json::{json, Value};
use std::{
    io::{self, BufRead, Read, Write},
    net::TcpStream,
};
fn emit(v: Value) {
    println!("{v}");
    io::stdout().flush().unwrap();
}
fn post(url: &str, body: Value) -> Value {
    let rest = url.strip_prefix("http://").unwrap();
    let (host, path) = rest.split_once('/').unwrap();
    let mut stream = TcpStream::connect(host).unwrap();
    let body = body.to_string();
    write!(stream, "POST /{path} HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
}
fn main() {
    if [
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_CODE_CHILD_SESSION",
        "CLAUDE_CODE_REMOTE_SESSION_ID",
    ]
    .iter()
    .any(|key| std::env::var_os(key).is_some())
    {
        std::process::exit(3);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version") {
        println!("fake-claude 1");
        return;
    }
    if args
        .windows(2)
        .any(|a| a[0] == "--model" && a[1] == "no-init")
    {
        std::thread::sleep(std::time::Duration::from_secs(10));
        return;
    }
    let settings: Value =
        serde_json::from_str(&args[args.iter().position(|a| a == "--settings").unwrap() + 1])
            .unwrap();
    let hook = |name: &str| {
        settings["hooks"][name][0]["hooks"][0]["url"]
            .as_str()
            .unwrap()
    };
    let id = args
        .iter()
        .position(|a| a == "--resume")
        .map(|i| args[i + 1].clone())
        .unwrap_or_else(|| format!("fake-{}", std::process::id()));
    emit(json!({"type":"system","subtype":"init","session_id":id}));
    for line in io::stdin().lock().lines() {
        let input: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let text = input["message"]["content"][0]["text"]
            .as_str()
            .unwrap_or("");
        if text == "exit" {
            return;
        }
        if text == "bad-json" {
            println!("{{not-json");
            io::stdout().flush().unwrap();
            return;
        }
        emit(json!({"type":"stream_event","event":{"delta":{"text":text}}}));
        emit(
            json!({"type":"assistant","message":{"content":[{"type":"text","text":"reply"},{"type":"tool_use","id":"tool-1","name":"Bash","input":{"command":"touch file"}}]}}),
        );
        let decision = post(
            hook("PermissionRequest"),
            json!({"tool_name":"Bash","tool_input":{"command":"touch file"}}),
        );
        let allowed = decision
            .pointer("/hookSpecificOutput/decision/behavior")
            .and_then(Value::as_str)
            == Some("allow");
        emit(
            json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"tool-1","is_error":!allowed,"content":if allowed {"allowed"} else {"denied"}}]}}),
        );
        let stop = post(hook("Stop"), json!({"stop_hook_active":false}));
        if stop["decision"] == "block" {
            emit(
                json!({"type":"assistant","message":{"content":[{"type":"text","text":stop["reason"]}]}}),
            );
        }
        emit(
            json!({"type":"result","subtype":"success","session_id":id,"result":"done","is_error":false,"usage":{"input_tokens":1,"output_tokens":2}}),
        );
    }
}
