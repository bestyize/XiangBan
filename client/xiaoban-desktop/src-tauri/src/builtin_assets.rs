//! Large built-in models are delivered by the resource server and cached locally.
use crate::library::{self, Resource};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path, sync::{LazyLock, Mutex}};
use std::time::Duration;

#[derive(Deserialize)]
struct Model {
    resource: Resource,
}

static ON_DEMAND: LazyLock<Vec<(Resource, Mutex<()>)>> = LazyLock::new(|| {
    let models: BTreeMap<String, Model> =
        serde_json::from_str(include_str!("../../builtin-models.json")).expect("valid built-in model manifest");
    models.into_values().map(|model| (model.resource, Mutex::new(()))).collect()
});

pub fn is_on_demand(hash: &str) -> bool {
    ON_DEMAND.iter().any(|(resource, _)| resource.sha256 == hash)
}

pub fn ensure_on_demand(root: &Path, hash: &str) -> Result<(), String> {
    let Some((resource, lock)) = ON_DEMAND.iter().find(|(resource, _)| resource.sha256 == hash) else {
        return Ok(());
    };
    let _guard = lock.lock().map_err(|_| "模型缓存暂不可用")?;
    library::download(root, resource, |_| {})
}

fn resources() -> [Resource; 2] {
    [
        Resource {
            url: "https://res.xiangban.xyz/files/7b68680fe1757d50cd22ece7a157dc43.glb".into(),
            sha256: "bda5a32ecd70e6247e0b6946db7e1db4e2393e32aacad9416597f2dac8d714ad".into(),
            bytes: 16_247_268,
            mime: "application/octet-stream".into(),
        },
        Resource {
            url: "https://res.xiangban.xyz/files/892a1cdb58ffe09ae4b5d10582d0b8f8.glb".into(),
            sha256: "67368dde88ab28b064e74bdd83e04bb9233f3fcbaef572a4438c8c180fc1d410".into(),
            bytes: 7_887_380,
            mime: "application/octet-stream".into(),
        },
    ]
}

pub fn find(hash: &str) -> Option<Resource> {
    ON_DEMAND.iter().find(|(resource, _)| resource.sha256 == hash).map(|(resource, _)| resource.clone())
        .or_else(|| resources().into_iter().find(|resource| resource.sha256 == hash))
}

pub fn start(app: &tauri::AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let mut pending = resources().into_iter().collect::<Vec<_>>();
        let delays = [20, 60, 300, 900];
        let mut attempt = 0usize;
        while !pending.is_empty() {
            if let Ok(root) = crate::content::root(&app) {
                pending.retain(|resource| library::download(&root, resource, |_| {}).is_err());
            }
            if pending.is_empty() { break; }
            std::thread::sleep(Duration::from_secs(delays[attempt.min(delays.len() - 1)]));
            attempt += 1;
        }
    });
}

#[cfg(test)]
#[test]
#[ignore = "downloads the published on-demand models"]
fn published_models_download_to_the_verified_cache() {
    let root = std::env::temp_dir().join(format!("xiaoban-published-models-{}", std::process::id()));
    for (resource, _) in ON_DEMAND.iter() {
        let path = root.join("cache").join(&resource.sha256);
        assert!(!path.exists());
        ensure_on_demand(&root, &resource.sha256).unwrap();
        assert!(library::verify_file(&path, resource).unwrap());
    }
    std::fs::remove_dir_all(root).unwrap();
}
