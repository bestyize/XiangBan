use std::{path::Path, time::Duration};
use xiaoban_agent_bridge::{
    classify_risk, claude::ClaudeCode, codex::Codex, BridgeError, ExternalAgent, PermissionProfile,
    Risk, StartRequest,
};
fn fake() -> Codex {
    Codex::new(env!("CARGO_BIN_EXE_fake-codex")).with_timeout(Duration::from_millis(150))
}
#[tokio::test]
async fn codex_lists_and_fails_closed_on_unverified_profiles() {
    let agent = fake();
    assert!(agent.availability().await.installed);
    assert_eq!(agent.list_sessions(2).await.unwrap()[0].id, "existing");
    let cwd = std::env::current_dir().unwrap();
    let result = agent
        .start(StartRequest {
            cwd,
            prompt: "hello".into(),
            profile: PermissionProfile::Strict,
            model: None,
            name: None,
        })
        .await;
    assert!(matches!(result, Err(BridgeError::Unsupported)));
}
#[tokio::test]
async fn request_timeout_is_reported() {
    assert!(matches!(
        fake().list_sessions(999).await,
        Err(BridgeError::Timeout)
    ));
}
#[tokio::test]
async fn claude_reports_a_missing_executable_and_rejects_relative_directories() {
    let a = ClaudeCode::new("missing-claude");
    let result = a
        .resume(
            "x",
            &std::env::current_dir().unwrap(),
            PermissionProfile::Strict,
        )
        .await;
    assert!(matches!(result, Err(BridgeError::NotInstalled)));
    let relative = a
        .resume("x", Path::new("relative"), PermissionProfile::Strict)
        .await;
    assert!(matches!(relative, Err(BridgeError::InvalidArgument)));
}
#[test]
fn risk_examples() {
    let cwd = Path::new("/work");
    for (s, risk) in [
        ("echo hello", Risk::Low),
        ("rm -rf stuff", Risk::High),
        ("npm install package", Risk::High),
        ("curl https://example.com", Risk::High),
        ("edit /outside/file", Risk::High),
        ("echo text > file", Risk::High),
        ("write ../outside", Risk::High),
        (r#"{"file_path":"/outside/file"}"#, Risk::High),
    ] {
        assert_eq!(classify_risk(s, cwd), risk, "{s}");
    }
}
