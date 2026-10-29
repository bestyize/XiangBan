//! Product policy only. The kernel owns generic discovery, never desktop tool names.
pub(super) fn defer_desktop_tool(name: &str) -> bool {
    // Web retrieval is a common path even when provided by the desktop adapter.
    !matches!(name, "web_search" | "web_fetch" | "web_extract" | "connector_search" | "connector_request")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_web_retrieval_stays_eager_and_desktop_actions_are_deferred() {
        for name in ["web_search", "web_fetch", "web_extract"] { assert!(!defer_desktop_tool(name)); }
        for name in ["wallpaper_set", "widget_add", "pet_update", "care_set", "desktop_future_tool"] { assert!(defer_desktop_tool(name)); }
    }
}
