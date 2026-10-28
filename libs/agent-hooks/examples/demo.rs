//! Print events until Ctrl+C or process termination.
use xiaoban_agent_hooks::receiver::{HookServer, ReceiverConfig};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = HookServer::start(ReceiverConfig::default()).await?;
    let mut events = server.events();
    println!("Listening on loopback port {}", server.endpoint().port);
    while let Ok(event) = events.recv().await {
        println!("{:?}", event);
    }
    server.shutdown().await;
    Ok(())
}
