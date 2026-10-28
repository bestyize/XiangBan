#![cfg(feature = "stdio")]
mod common;
use common::{Fake, TestServer, TOKEN};
use serde_json::json;
use std::{sync::{atomic::Ordering, Arc}, time::Duration};

/// Explicit opt-in: uses the installed Claude Code account and makes a real model request.
#[tokio::test]
#[ignore = "requires installed, authenticated Claude Code and network access"]
async fn claude_code_calls_voice_tools_via_stdio() {
    let fake = Arc::new(Fake::default());
    let server = TestServer::with(fake.clone(), Default::default(), true).await;
    let work = std::env::current_dir().unwrap().join("target/claude-code-integration");
    std::fs::create_dir_all(&work).unwrap();
    let config = work.join("mcp.json");
    let content = json!({"mcpServers":{"xiaoban_voice":{
        "command":env!("CARGO_BIN_EXE_xiaoban-voice-mcp-stdio"),
        "env":{"XIAOBAN_VOICE_URL":format!("http://{}", server.running.addr), "XIAOBAN_VOICE_TOKEN":TOKEN}
    }}});
    std::fs::write(&config, serde_json::to_vec(&content).unwrap()).unwrap();
    #[cfg(windows)]
    let mut cmd = { let mut c = tokio::process::Command::new("cmd.exe"); c.args(["/d", "/c", "claude.cmd"]); c };
    #[cfg(not(windows))]
    let mut cmd = tokio::process::Command::new("claude");
    let output = tokio::time::timeout(Duration::from_secs(120), cmd.current_dir(&work)
        .args(["--print", "Use the xiaoban_voice speak tool with text 'integration test', then the listen tool. Report the tool results.",
            "--strict-mcp-config", "--mcp-config"]).arg(&config)
        .args(["--allowedTools", "mcp__xiaoban_voice__speak", "mcp__xiaoban_voice__listen", "--no-session-persistence", "--output-format", "json"])
        .kill_on_drop(true).output()).await.expect("Claude Code timed out").expect("Claude Code failed to start");
    std::fs::write(work.join("claude-stdout.json"), &output.stdout).unwrap();
    std::fs::write(work.join("claude-stderr.txt"), &output.stderr).unwrap();
    assert!(output.status.success(), "Claude Code exited {}: stdout={}, stderr={}", output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(fake.played.load(Ordering::SeqCst) > 0, "Claude did not invoke speak: {}", String::from_utf8_lossy(&output.stdout));
    assert!(fake.mic_entered.load(Ordering::SeqCst) > 0, "Claude did not invoke listen: {}", String::from_utf8_lossy(&output.stdout));
}
