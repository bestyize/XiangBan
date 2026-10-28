use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
fn emit(v: Value) {
    println!("{v}");
    io::stdout().flush().unwrap();
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

    if std::env::args().any(|a| a == "--version") {
        println!("fake-codex 1");
        return;
    }
    let mut linger = false;
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let v: Value = serde_json::from_str(&line).unwrap();
        let id = &v["id"];
        match v["method"].as_str().unwrap_or("") {
            "initialize" => {
                emit(json!({"method":"configWarning","params":{}}));
                emit(json!({"id":id,"result":{}}));
            }
            "thread/list" => {
                if v["params"]["limit"] != 999 {
                    emit(
                        json!({"id":id,"result":{"data":[{"id":"existing","cwd":"/tmp","updatedAt":1}]}}),
                    );
                }
            }
            "thread/start" | "thread/resume" => {
                emit(json!({"id":id,"result":{"thread":{"id":"test-thread"}}}))
            }
            "turn/start" => {
                let prompt = v["params"]["input"][0]["text"].as_str().unwrap_or("");
                if prompt == "linger" {
                    linger = true;
                }
                if prompt == "timeout" {
                    continue;
                }
                if prompt == "exit" {
                    return;
                }
                if prompt == "bad-json" {
                    emit(json!({"id":id,"result":{"turn":{"id":"turn-1"}}}));
                    println!("{{invalid");
                    io::stdout().flush().unwrap();
                    return;
                }
                emit(json!({"method":"turn/started","params":{"turn":{"id":"turn-1"}}}));
                emit(json!({"id":id,"result":{"turn":{"id":"turn-1"}}}));
                emit(json!({"method":"item/agentMessage/delta","params":{"delta":"hello"}}));
                if prompt == "unsupported" {
                    emit(json!({"id":92,"method":"account/chatgptAuthTokens/refresh","params":{}}));
                }
                emit(
                    json!({"id":91,"method":"item/commandExecution/requestApproval","params":{"command":"echo ok"}}),
                );
                if prompt == "resolved" {
                    emit(json!({"method":"serverRequest/resolved","params":{"requestId":91}}));
                }
            }
            "turn/steer" | "turn/interrupt" | "thread/name/set" => {
                emit(json!({"id":id,"result":{}}))
            }
            _ => {
                if id == 92 && v["error"]["code"] == -32601 {
                    emit(
                        json!({"method":"item/agentMessage/delta","params":{"delta":"unsupported-rejected"}}),
                    );
                }
                if id == 91 {
                    let allowed = matches!(
                        v["result"]["decision"].as_str(),
                        Some("accept" | "acceptForSession")
                    );
                    emit(
                        json!({"method":"item/completed","params":{"item":{"type":"commandExecution","id":"command-1","status":if allowed {"completed"} else {"failed"},"aggregatedOutput":if allowed {"allowed"} else {"denied"}}}}),
                    );
                    emit(json!({"method":"turn/completed","params":{"turn":{"id":"turn-1"}}}));
                }
            }
        }
    }
    if linger {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
