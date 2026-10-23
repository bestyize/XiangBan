//! xiaoban-infer: 小伴桌面的本地推理引擎. One process serves one capability, chosen at start:
//!
//! `xiaoban-infer --capability asr --dir <models>`, or `--capability turn`.
//!
//! Capabilities share the line protocol in `protocol.rs` and the onnxruntime that sherpa-onnx links in. Adding one
//! means a module with `load` and `handle`, and a line in `start` below.
mod asr;
mod features;
mod protocol;
mod turn;

use protocol::{send, Request, Response, PROTOCOL_VERSION};
use std::{env, io::{self, BufRead}, path::PathBuf};

/// A loaded capability.
enum Capability { Asr(asr::Asr), Turn(turn::Turn) }

impl Capability {
    fn handle(&mut self, request: Request) -> Response {
        match self { Self::Asr(asr) => asr.handle(request), Self::Turn(turn) => turn.handle(request) }
    }
}

fn start(name: &str, directory: &PathBuf) -> Result<(Capability, &'static str), String> {
    match name {
        "asr" => asr::Asr::load(directory).map(|c| (Capability::Asr(c), "asr")),
        "turn" => turn::Turn::load(directory).map(|c| (Capability::Turn(c), "turn")),
        other => Err(format!("未知的能力：{other}")),
    }
}

fn argument(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

fn model_dir() -> PathBuf {
    if let Some(dir) = argument("--dir").or_else(|| argument("-d")) { return PathBuf::from(dir); }
    env::current_exe().ok().and_then(|exe| exe.parent().map(PathBuf::from)).unwrap_or_else(|| PathBuf::from("."))
}

fn main() {
    let directory = model_dir();
    let Some(name) = argument("--capability") else {
        send(&Response::error("请用 --capability 指定要提供的能力（asr、turn）"));
        std::process::exit(2);
    };
    let (mut capability, served) = match start(&name, &directory) {
        Ok(started) => started,
        Err(error) => { send(&Response::error(error)); std::process::exit(1); }
    };
    send(&Response::Ready { version: PROTOCOL_VERSION, capabilities: vec![served] });
    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() { continue; }
        let response = match serde_json::from_str::<Request>(line.trim()) {
            Err(e) => Response::error(format!("无效请求 JSON：{e}")),
            Ok(Request::Exit) => break,
            Ok(Request::Ping) => Response::Pong,
            Ok(request) => capability.handle(request),
        };
        send(&response);
    }
}
