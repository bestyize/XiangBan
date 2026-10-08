use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
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

pub async fn static_handler(uri: Uri) -> Response {
    let mut path = uri.path().trim_start_matches('/').to_string();
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
