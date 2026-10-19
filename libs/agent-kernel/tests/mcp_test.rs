#![cfg(feature = "mcp")]
mod common;

use agent_kernel::mcp::{McpClient, McpConfig};
use agent_kernel::provider::scripted::Step;
use agent_kernel::*;
use common::*;
use std::fs;
use std::sync::Arc;
use serde_json::json;

fn fake_mcp_server_script() -> TempDir {
    let dir = TempDir::new("mcp-fake");
    let script_path = dir.path("mcp_server.py");
    let py = r#"
import sys, json, os

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
            if text_val == "rich":
                resp["result"] = {"content": [{"type":"text", "text":"screen"}, {"type":"image", "mimeType":"image/png", "data":"AAAA"}], "structuredContent":{"window":"test"}, "isError": True}
            elif text_val == "environment":
                resp["result"] = {"structuredContent": {key:os.environ.get(key) for key in ["KERNEL_TEST_SECRET", "KERNEL_TEST_EXPLICIT", "PATH"]}}
            elif text_val == "bad_image":
                resp["result"] = {"content":[{"type":"image","mimeType":"image/png","data":"not base64!"}]}
            elif text_val == "unsupported":
                resp["result"] = {"content":[{"type":"audio","data":"AAAA"}]}
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
    assert!(tools.iter().all(|tool| tool.spec().untrusted_output));

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

#[test]
fn mcp_media_json_and_error_flags_reach_the_harness_and_history() {
    let temp = fake_mcp_server_script();
    let client = McpClient::spawn(McpConfig::new("python3", &["-u", &temp.path("mcp_server.py").display().to_string()])).unwrap();
    let store=Arc::new(MemoryStore::new());
    let provider=Arc::new(agent_kernel::provider::scripted::ScriptedProvider::new(vec![Step::tool_call("c", "echo_test", r#"{"text":"rich"}"#), Step::text("done")]));
    let mut builder=Agent::builder(provider.clone(),store.clone());
    for tool in client.tools() { builder=builder.tool_arc(tool); }
    run(&builder.build().unwrap(),"look").0.unwrap();
    let result=store.load("s1").unwrap().into_iter().find(|m|m.role==Role::Tool).unwrap();
    assert!(result.is_error);
    assert_eq!(result.structured_content,Some(json!({"window":"test"})));
    assert!(result.parts.contains(&Part::Image{url:"data:image/png;base64,AAAA".into()}));
    assert_eq!(provider.requests()[1].messages.iter().find(|m|m.role==Role::Tool),Some(&result));
    for text in ["bad_image","unsupported"] {
        assert!(client.call_tool_result("echo_test",json!({"text":text}),&ToolCtx::detached("s")).is_err());
    }
}

#[test]
fn mcp_environment_is_allowlisted() {
    // Run in an isolated test process, so no test mutates the parent process's global environment.
    if std::env::var("KERNEL_TEST_SECRET").as_deref()!=Ok("sentinel-secret") {
        let output=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","mcp_environment_is_allowlisted","--nocapture"])
            .env("KERNEL_TEST_SECRET","sentinel-secret").output().unwrap();
        assert!(output.status.success(), "{}{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
        return;
    }
    let temp=fake_mcp_server_script();
    let script=temp.path("mcp_server.py").display().to_string();
    for inherit in [false,true] {
        let mut config=McpConfig::new("python3",&["-u",&script]).env("KERNEL_TEST_EXPLICIT","configured");
        if inherit { config=config.inherit_env("KERNEL_TEST_SECRET"); }
        let client=McpClient::spawn(config).unwrap();
        let result=client.call_tool_result("echo_test",json!({"text":"environment"}),&ToolCtx::detached("s")).unwrap();
        let env=result.structured_content.unwrap();
        assert_eq!(env["KERNEL_TEST_EXPLICIT"],"configured");
        assert_eq!(env["KERNEL_TEST_SECRET"], if inherit {json!("sentinel-secret")} else {serde_json::Value::Null});
        assert!(env["PATH"].is_string());
    }
}
