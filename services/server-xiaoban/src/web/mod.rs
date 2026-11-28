use axum::{
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "web/dist/"]
pub struct Asset;

fn cache_control(path: &str) -> &'static str {
    // Vite emits content-hashed files under assets/; everything else (index.html, sw.js, manifest) must revalidate.
    if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    }
}

fn is_mobile_ua(headers: &HeaderMap) -> bool {
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let lower = ua.to_ascii_lowercase();
    lower.contains("android")
        || lower.contains("iphone")
        || lower.contains("ipad")
        || lower.contains("ipod")
        || lower.contains("mobile")
}

fn legacy_redirect(path: &str, query: Option<&str>, headers: &HeaderMap) -> Option<Response> {
    let normalized = path.trim_end_matches('/').to_ascii_lowercase();
    let suffix = query.filter(|q| !q.is_empty()).map(|q| format!("?{q}")).unwrap_or_default();
    match normalized.as_str() {
        "desktop" | "pc" | "features" | "desktop-experience" => {
            Some(Redirect::permanent(&format!("https://xiaban.xyz/desktop{suffix}")).into_response())
        }
        "app" | "mobile" => {
            Some(Redirect::permanent(&format!("https://xiaban.xyz/app{suffix}")).into_response())
        }
        "download" => {
            let target = if is_mobile_ua(headers) {
                format!("https://xiaban.xyz/app{suffix}")
            } else {
                format!("https://xiaban.xyz/desktop{suffix}")
            };
            Some(Redirect::temporary(&target).into_response())
        }
        "plugins" => Some(Redirect::temporary("https://market.xiangban.xyz/").into_response()),
        _ => None,
    }
}

pub async fn static_handler(uri: Uri, headers: HeaderMap) -> Response {
    let raw_path = uri.path().trim_start_matches('/');
    if let Some(resp) = legacy_redirect(raw_path, uri.query(), &headers) {
        return resp;
    }

    let mut path = raw_path.to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    if let Some(content) = Asset::get(&path) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        return (
            [
                (header::CONTENT_TYPE, mime.as_ref()),
                (header::CACHE_CONTROL, cache_control(&path)),
            ],
            content.data,
        )
            .into_response();
    }

    let looks_like_file = path.rsplit('/').next().is_some_and(|name| name.contains('.'));
    if looks_like_file || path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
    }

    // SPA fallback: client-side routes are served index.html
    match Asset::get("index.html") {
        Some(content) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            content.data,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "Frontend not built. Run `npm install && npm run build` in web/ and restart.",
        )
            .into_response(),
    }
}
