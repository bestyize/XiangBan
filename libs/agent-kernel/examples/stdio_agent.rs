//! Reference host: speaks the JSON-lines protocol of `agent_kernel::stdio` on stdin/stdout.
//!
//! AGENT_BASE_URL, AGENT_API_KEY and AGENT_MODEL select the endpoint. AGENT_PROTOCOL picks the wire
//! format: `chat` (default, /chat/completions), `responses` (/responses) or `anthropic` (/v1/messages).
//! Optional: AGENT_MAX_TOKENS, AGENT_SYSTEM_PROMPT, AGENT_SESSIONS_DIR (default ./.agent-sessions),
//! AGENT_ROOT (default .).
//! File tools are read-only inside AGENT_ROOT; anything that changes state asks the client first.
//! With feature `process`, AGENT_ALLOW_PROCESSES=1 explicitly enables managed process tools.

use agent_kernel::builtin::{fs::fs_tools, AskUser, TimeNow, Todo};
use agent_kernel::provider::anthropic::AnthropicMessages;
use agent_kernel::provider::openai::OpenAiCompat;
use agent_kernel::provider::responses::OpenAiResponses;
use agent_kernel::provider::wire::ProviderProfile;
use agent_kernel::stdio::StdioBridge;
use agent_kernel::*;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = env("AGENT_BASE_URL").ok_or("AGENT_BASE_URL is required")?;
    let model = env("AGENT_MODEL").ok_or("AGENT_MODEL is required")?;
    let mut profile = ProviderProfile::new(base_url, env("AGENT_API_KEY").unwrap_or_default(), model);
    profile.max_output_tokens = env("AGENT_MAX_TOKENS").map(|v| v.parse()).transpose()?;
    let provider: Arc<dyn ModelProvider> = match env("AGENT_PROTOCOL").as_deref().unwrap_or("chat") {
        "chat" => Arc::new(OpenAiCompat::new(profile)?),
        "responses" => Arc::new(OpenAiResponses::new(profile)?),
        "anthropic" => Arc::new(AnthropicMessages::new(profile)?),
        other => return Err(format!("AGENT_PROTOCOL must be chat, responses or anthropic, not `{other}`").into()),
    };
    let root = PathBuf::from(env("AGENT_ROOT").unwrap_or_else(|| ".".into()));
    let sessions = PathBuf::from(env("AGENT_SESSIONS_DIR").unwrap_or_else(|| ".agent-sessions".into()));

    let bridge = StdioBridge::new(io::stdout());
    let mut builder = Agent::builder(provider, Arc::new(JsonlStore::new(sessions)))
        .system_prompt(env("AGENT_SYSTEM_PROMPT").unwrap_or_default())
        .tool(TimeNow::new())
        .tool(Todo::new())
        .tool(AskUser::new(bridge.ask_handler()))
        .middleware(Permission::new(PermissionMode::Ask, bridge.approver()))
        .middleware(ContextTrim::new(100_000).reserve_output(8192));
    #[cfg(feature = "process")]
    if env("AGENT_ALLOW_PROCESSES").as_deref() == Some("1") {
        let processes = Arc::new(agent_kernel::process::ProcessManager::new(&root)?);
        for tool in processes.tools() { builder = builder.tool_arc(tool); }
    }
    for tool in fs_tools(vec![root])? {
        builder = builder.tool_arc(tool);
    }
    bridge.serve(Arc::new(builder.build()?), BufReader::new(io::stdin().lock()))?;
    Ok(())
}
