#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod links;

use links::{classify, Navigation};
use tauri::{webview::NewWindowResponse, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use url::Url;

const WINDOW_LABEL: &str = "main";

fn open_externally(app: &AppHandle, url: &Url) {
    if let Err(err) = app.opener().open_url(url.as_str(), None::<&str>) {
        eprintln!("无法用系统程序打开 {url}: {err}");
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let on_navigation_app = app.handle().clone();
            let on_new_window_app = app.handle().clone();
            // The window starts on the bundled launcher page, which moves on to the site once it is reachable.
            WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App("index.html".into()))
                .title("小伴")
                .inner_size(480.0, 860.0)
                .min_inner_size(360.0, 600.0)
                .center()
                .on_navigation(move |url| match classify(url) {
                    Navigation::Stay => true,
                    Navigation::OpenExternally => {
                        open_externally(&on_navigation_app, url);
                        false
                    }
                    Navigation::Block => false,
                })
                .on_new_window(move |url, _features| {
                    if classify(&url) == Navigation::OpenExternally {
                        open_externally(&on_new_window_app, &url);
                    }
                    NewWindowResponse::Deny
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("小伴桌面壳启动失败");
}
