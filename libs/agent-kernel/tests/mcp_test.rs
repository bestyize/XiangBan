#![cfg(feature = "mcp")]
mod common;

use agent_kernel::mcp::{McpClient, McpConfig};
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::fs;
use std::sync::Arc;

fn fake_mcp_server_script() -> TempDir {
    let dir = TempDir::new("mcp-fake");
    let script_path = dir.path("mcp_server.py");
    let py = r#"
import sys, json

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    req = json.loads(line)
    method = req.get("method")
    req_id = req.get("id")

    if method == "initialize":
        resp = {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "serverInfo": {"name": "fake-server", "version": "1.0.0"}
            }
        }
        print(json.dumps(resp), flush=True)
    elif method == "notifications/initialized":
        pass
    elif method == "tools/list":
        resp = {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": [
                    {
                        "name": "echo_test",
                        "description": "Echoes back the test input",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"text": {"type": "string"}},
                            "required": ["text"]
                        }
                    },
                    {
                        "name": "fail_test",
                        "description": "Fails on demand",
                        "inputSchema": {"type": "object"}
                    }
                ]
            }
        }
        print(json.dumps(resp), flush=True)
    elif method == "tools/call":
        params = req.get("params", {})
        tool_name = params.get("name")
        args = params.get("arguments", {})
        if tool_name == "echo_test":
            text_val = args.get("text", "")
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [{"type": "text", "text": f"echoed: {text_val}"}],
                    "isError": False
                }
            }
        elif tool_name == "fail_test":
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [{"type": "text", "text": "explicit error reported"}],
                    "isError": True
                }
            }
        else:
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "error": {"code": -32601, "message": "Method not found"}
            }
        print(json.dumps(resp), flush=True)
"#;
    fs::write(&script_path, py).unwrap();
    dir
}

#[test]
fn test_mcp_handshake_and_tool_execution() {
    let temp = fake_mcp_server_script();
    let script = temp.path("mcp_server.py");
    let script_str = script.display().to_string();

    let config = McpConfig::new("python3", &["-u", &script_str])
        .tool_prefix("mcp_")
        .default_risk(Risk::Write);

    let client = McpClient::spawn(config).expect("mcp client spawns and completes handshake");
    let tools = client.tools();

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].spec().name, "mcp_echo_test");
    assert_eq!(tools[1].spec().name, "mcp_fail_test");
    assert_eq!(tools[0].spec().risk, Risk::Write);

    // Run in Agent harness
    let steps = vec![
        Step::tool_call("call_1", "mcp_echo_test", r#"{"text": "hello mcp"}"#),
        Step::tool_call("call_2", "mcp_fail_test", "{}"),
        Step::text("all completed"),
    ];

    let mut builder = Agent::builder(
        Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(steps)),
        Arc::new(MemoryStore::new()),
    );
    for t in tools {
        builder = builder.tool_arc(t);
    }
    let agent = builder.build().unwrap();

    let res = agent.run("s1", "go", &mut |_| {}, &Arc::new(std::sync::atomic::AtomicBool::new(false)));
    let summary = res.expect("agent runs successfully");
    assert_eq!(summary.text, "all completed");
}
