//! Opt-in integration test: isolated config, real Claude Code and explicit API key.
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};
use xiaoban_agent_hooks::{
    claude_settings::{apply, plan_install, InstallSpec, Scope},
    events::HookDecision,
    receiver::{HookServer, ReceiverConfig},
};

struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires XIAOBAN_REAL_CLAUDE=1 and an explicitly supplied ANTHROPIC_API_KEY"]
async fn real_claude_permission_stop_and_queued_instruction() {
    assert_eq!(std::env::var("XIAOBAN_REAL_CLAUDE").as_deref(), Ok("1"));
    assert!(
        std::env::var_os("ANTHROPIC_API_KEY").is_some(),
        "isolated configuration requires ANTHROPIC_API_KEY"
    );
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    let config = dir.path().join("claude-config");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(&config).unwrap();
    let endpoint = dir.path().join("hook-endpoint.json");
    std::env::set_var("XIAOBAN_HOOK_ENDPOINT", &endpoint);
    let mut spec = InstallSpec::new(PathBuf::from(env!("CARGO_BIN_EXE_xiaoban-hook")));
    spec.events = vec!["PermissionRequest".into(), "Stop".into()];
    let plan = plan_install(&Scope::Project(project.clone()), &spec).unwrap();
    apply(&plan).unwrap();
    let server = HookServer::start(ReceiverConfig::default()).await.unwrap();
    let mut events = server.events();
    let executable = std::env::var_os("XIAOBAN_CLAUDE_BIN").unwrap_or_else(|| "claude".into());
    let process=Command::new(executable).current_dir(&project).env("CLAUDE_CONFIG_DIR",&config).env("XIAOBAN_HOOK_ENDPOINT",&endpoint)
        .args(["--print","--no-session-persistence","--permission-mode","default","--tools","Bash","--strict-mcp-config","--setting-sources",""])
        .arg("--settings").arg(&plan.path).arg("Using the Bash tool, run exactly `touch first.txt` in the current directory, then finish.")
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("start Claude Code; use XIAOBAN_CLAUDE_BIN if needed");
    let mut process = KillOnDrop(process);
    let mut saw_permission = false;
    let mut stops = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), events.recv()).await {
            Ok(Ok(event)) if event.hook_event_name == "PermissionRequest" => {
                if !saw_permission {
                    server.queue_instruction(&event.session_id,"Use the Bash tool to run exactly `touch second.txt` in the current directory, then finish.".into());
                }
                saw_permission = true;
                server.respond(event.id, HookDecision::Allow);
            }
            Ok(Ok(event)) if event.hook_event_name == "Stop" => {
                stops += 1;
            }
            _ => {}
        }
        if saw_permission
            && stops >= 2
            && project.join("first.txt").exists()
            && project.join("second.txt").exists()
        {
            break;
        }
        if process.0.try_wait().unwrap().is_some() {
            break;
        }
    }
    server.shutdown().await;
    assert!(saw_permission, "PermissionRequest was not observed");
    assert!(stops >= 2, "Stop and continuation were not both observed");
    assert!(
        project.join("first.txt").exists(),
        "approved tool did not create first file"
    );
    assert!(
        project.join("second.txt").exists(),
        "queued instruction did not create second file"
    );
}
