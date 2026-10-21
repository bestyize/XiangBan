//! Live API verification: cargo run --example catalog-smoke -- [pack-slug] [--install]
//! --install uses a temporary workspace, verifies readiness, then removes it. No scripts run.
use agent_skill_discovery::{catalog, packs, SkillDiscovery};
struct Temporary(std::path::PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    let reviewed = args
        .windows(2)
        .find(|a| a[0] == "--reviewed-sha")
        .map(|a| a[1].clone());
    if reviewed.is_none() {
        let experts = catalog::browse("experts", "", 1, "")?;
        let connectors = catalog::browse("connectors", "", 1, "")?;
        println!(
            "SkillHub: {} expert packs, {} MCP services",
            experts["total"], connectors["total"]
        );
        let detail = catalog::connector("qcc-company-basic-information-mcp")?;
        println!(
            "MCP README verified; {} connection templates",
            detail["configs"].as_array().map(Vec::len).unwrap_or(0)
        );
    }
    if let Some(slug) = std::env::args().nth(1) {
        let root = Temporary(std::env::temp_dir().join(format!(
                "xiaoban-catalog-smoke-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        let discovery = SkillDiscovery::new(root.0.join("skills"), root.0.clone());
        let sha = if let Some(sha) = reviewed {
            sha
        } else {
            let review = packs::inspect(&discovery, &slug)?;
            println!(
                "Reviewed {}: {} pinned dependencies; SHA256={}",
                review["slug"],
                review["dependencies"].as_array().unwrap().len(),
                review["sha256"]
            );
            review["sha256"].as_str().unwrap().to_string()
        };
        if std::env::args().any(|arg| arg == "--install") {
            let receipts = root.0.join("packs");
            packs::install(&discovery, &receipts, &slug, &sha)?;
            let installed = packs::installed(&root.0.join("skills"), &receipts)?;
            if installed[0]["ready"] != true {
                return Err("installed workflow or dependencies not ready".into());
            }
            packs::uninstall(&root.0.join("skills"), &receipts, &slug)?;
            println!("Installed and verified all dependencies; removed workflow while preserving shared skills.");
        }
    }
    Ok(())
}
