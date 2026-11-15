//! Large built-in models are delivered by the resource server and cached locally.
use crate::library::{self, Resource};
use std::time::Duration;

fn resources() -> [Resource; 2] {
    [
        Resource {
            url: "https://res.xiangban.xyz/files/5b97664f1eb0e64a3253c5f38e866d14.glb".into(),
            sha256: "db223def3a0d109110b8a27901b3ae52157c1961bd2d11b972b89ec203bc6cb6".into(),
            bytes: 15_033_348,
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
    resources().into_iter().find(|resource| resource.sha256 == hash)
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
