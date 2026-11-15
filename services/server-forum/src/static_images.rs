use std::{collections::HashMap, sync::OnceLock};

/// Only paths of static files that shipped before the WebP conversion are aliases.
pub(crate) fn webp_path(path: &str) -> Option<&'static str> {
    static ALIASES: OnceLock<HashMap<String, String>> = OnceLock::new();
    ALIASES
        .get_or_init(|| {
            serde_json::from_str(include_str!("../web/static-image-aliases.json"))
                .expect("valid static image aliases")
        })
        .get(path)
        .map(String::as_str)
}
