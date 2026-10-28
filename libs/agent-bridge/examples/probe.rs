use xiaoban_agent_bridge::{claude::ClaudeCode, codex::Codex, ExternalAgent};
#[tokio::main]
async fn main() {
    for agent in [
        &Codex::default() as &dyn ExternalAgent,
        &ClaudeCode::default() as &dyn ExternalAgent,
    ] {
        let a = agent.availability().await;
        if !a.installed {
            println!("{:?}: \u{672a}\u{5b89}\u{88c5}", agent.kind());
            continue;
        }
        println!(
            "{:?}: {}",
            agent.kind(),
            a.version.as_deref().unwrap_or("unknown version")
        );
        match agent.list_sessions(5).await {
            Ok(s) => println!("  sessions: {}", s.len()),
            Err(e) => println!("  sessions unavailable: {e}"),
        }
    }
}
