//! Only host-created configuration surfaces may request this narrow set of console operations.
pub fn kind(label: &str) -> Option<&'static str> {
    if let Some(suffix) = label.strip_prefix("configuration-store-") {
        return (suffix.len() == 24 && suffix.bytes().all(|c| c.is_ascii_hexdigit())).then_some("store");
    }
    if let Some(suffix) = label.strip_prefix("configuration-cloud-") { return (suffix.len() == 24 && suffix.bytes().all(|c| c.is_ascii_hexdigit())).then_some("cloud"); }
    if label == "configuration-community" { return Some("community"); }
    if label == "configuration-settings" { return Some("settings"); }
    let suffix = label.strip_prefix("configuration-model-")?;
    (suffix.len() == 24 && suffix.bytes().all(|c| c.is_ascii_hexdigit())).then_some("model")
}
pub fn command_allowed(role: &str, command: &str) -> bool {
    if role == "cloud" { return ["get_settings", "library_cloud_item", "library_cloud_open_source", "library_cloud_download", "library_cloud_download_cancel", "library_cloud_read", "library_cloud_open_task"].contains(&command); }
    if role == "store" { return [
        "get_settings", "get_app_info", "open_download_url", "ai_store_open_task",
        "ai_store_installed", "ai_store_inspect", "ai_store_install", "ai_store_uninstall",
        "ai_store_experts_installed", "ai_store_expert_review", "ai_store_expert_install", "ai_store_expert_remove",
        "ai_store_connector_status", "ai_store_connector_detail", "ai_store_connector_connect", "ai_store_connector_disconnect", "ai_store_connector_reconnect",
        "ai_store_connector_auth_status", "ai_store_connector_auth_cancel",
        "ai_connector_installed", "ai_connector_configure", "ai_connector_set_enabled", "ai_connector_unbind",
        "ai_connector_baidu_start", "ai_connector_baidu_status", "ai_connector_baidu_cancel",
        "ai_market_detail", "ai_market_inspect", "ai_market_install", "ai_market_updates", "ai_market_update",
        "ai_market_group_inspect", "ai_market_group_install",
    ].contains(&command); }
    if role == "community" { return ["get_settings","community_browse","community_schemes_status","community_schemes_save","community_schemes_remove","community_schemes_restore","community_schemes_cloud_enabled","community_schemes_cloud","open_download_url"].contains(&command); }
    const COMMON: &[&str] = &[
        "get_settings", "get_app_info", "get_account_session", "get_website_status", "get_appearance",
        "get_ai_config", "refresh_ai_providers", "save_ai_provider", "delete_ai_provider", "switch_ai_provider",
        "save_ai_preferences", "fetch_ai_models", "pick_workspace", "open_download_url",
        "ai_gateway_balance", "ai_gateway_tiers", "ai_gateway_redeem", "ai_gateway_probe", "use_ai_gateway",
    ];
    const SETTINGS: &[&str] = &[
        "get_startup_status", "get_game_status", "update_settings", "set_autostart", "open_data_directory",
        "set_appearance", "set_accent", "set_chat_composer_pet", "open_application_skins", "set_automatic_updates", "check_website", "dismiss_website_message",
        "get_network_config", "save_network_config", "test_network_connection", "assistant_status", "assistant_configure",
        "assistant_try", "open_screen_capture_settings", "reset_screen_capture_permission",
        "chat_agent_settings", "chat_set_command_sandbox", "chat_set_sandbox_network", "chat_set_execution_limits",
        "chat_set_windows_sandbox", "chat_save_hooks", "chat_revoke_grant",
        "tts_get_settings", "tts_save_settings", "tts_list_voices", "tts_synthesize", "tts_synthesize_part", "tts_cancel",
        "asr_get_status", "asr_set_enabled", "asr_prepare", "asr_report_microphone",
        "send_account_code", "login_account_with_code", "login_account_with_password", "register_account",
        "reset_account_password", "check_account_username", "get_signup_methods", "logout_account",
        "phone_link_status", "phone_link_set_enabled", "phone_link_remove_peer",
        "speech_agent_env", "speech_agent_install_guides", "speech_agent_hooks_status", "speech_agent_hooks_install", "speech_agent_hooks_uninstall", "speech_agent_inject",
        "speech_agent_delegate", "speech_agent_task_status", "speech_agent_task_cancel", "speech_agent_task_list",
        "design_mcp_status", "design_mcp_set_enabled", "design_mcp_configure",
        "voice_service_status", "voice_service_set_enabled", "voice_service_reset_token",
    ];
    matches!(role, "settings" | "model") && (COMMON.contains(&command) || role == "settings" && SETTINGS.contains(&command))
}
/// Public catalog metadata only; configuration and credentials never enter window URLs.
pub fn store_identity(raw: &str) -> Result<String, String> {
    if raw.len() > 16_384 { return Err("插件信息过大".into()); }
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|_| "插件信息无效")?;
    let object = value.as_object().ok_or("插件信息无效")?;
    if object.keys().any(|k| !["view", "kind", "source", "item"].contains(&k.as_str())) { return Err("插件参数无效".into()); }
    let view = value["view"].as_str().filter(|v| ["skill", "registry", "connector", "group", "update"].contains(v)).ok_or("插件视图无效")?;
    let kind = value["kind"].as_str().filter(|v| ["skills", "experts", "connectors"].contains(v)).ok_or("插件类型无效")?;
    let source = value["source"].as_str().filter(|v| ["market", "hub"].contains(v)).ok_or("插件来源无效")?;
    let item = value["item"].as_object().ok_or("插件编号无效")?;
    if item.iter().any(|(k, v)| !["slug", "name", "description", "version", "namespace", "category", "iconUrl", "variant"].contains(&k.as_str()) || !(v.is_string() || v.is_null())) { return Err("插件仅允许目录信息".into()); }
    let slug = value["item"]["slug"].as_str().filter(|v| !v.is_empty() && v.len() <= 512 && !v.chars().any(char::is_control)).ok_or("插件编号无效")?;
    Ok(serde_json::json!([view, kind, source, value["item"]["namespace"], slug]).to_string())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn store_windows_accept_only_catalog_metadata_and_narrow_commands() {
        assert_eq!(kind("configuration-store-012345678901234567890123"), Some("store"));
        assert_eq!(kind("configuration-store-forged"), None);
        assert!(command_allowed("store", "ai_connector_configure"));
        assert!(command_allowed("store", "ai_store_open_task"));
        for command in ["chat_send", "library_remove", "get_account_session", "save_ai_provider", "save_network_config", "ai_market_take_link"] {
            assert!(!command_allowed("store", command));
        }
        let mut value = serde_json::json!({"view":"connector","kind":"connectors","source":"market","item":{"slug":"docs","name":"文档"}});
        let identity = store_identity(&value.to_string()).unwrap();
        value["item"]["name"] = serde_json::json!("更新后的名称");
        assert_eq!(identity, store_identity(&value.to_string()).unwrap());
        value["item"]["token"] = serde_json::json!("must-never-enter-the-url");
        assert!(store_identity(&value.to_string()).is_err());
        assert!(store_identity(r#"{"view":"unknown"}"#).is_err());
    }
    #[test] fn scope_is_closed_by_default() {
        assert_eq!(kind("configuration-settings"), Some("settings"));
        assert!(command_allowed("settings", "open_application_skins"));
        assert!(!command_allowed("model", "open_application_skins"));
        assert_eq!(kind("configuration-model-012345678901234567890123"), Some("model"));
        for label in ["wallpaper", "configuration-model-../control", "configuration-settings-other"] { assert!(kind(label).is_none()); }
        for role in ["settings", "model"] {
            assert!(!command_allowed(role, "chat_send"));
            assert!(!command_allowed(role, "library_remove"));
            assert!(!command_allowed(role, "configuration_window_reply"));
        }
        assert_eq!(kind("configuration-community"),Some("community"));
        assert!(command_allowed("community","community_schemes_status"));
        assert!(!command_allowed("community","get_account_session"));
        assert!(!command_allowed("community","chat_send"));
        assert!(!command_allowed("community","save_network_config"));
        assert!(command_allowed("settings", "design_mcp_configure"));
        assert!(!command_allowed("model", "design_mcp_configure"));
        assert!(!command_allowed("community", "design_mcp_configure"));
        assert!(command_allowed("settings", "save_network_config"));
        assert!(!command_allowed("model", "save_network_config"));
        assert!(command_allowed("settings", "set_chat_composer_pet"));
        assert!(!command_allowed("model", "set_chat_composer_pet"));
        assert!(command_allowed("model", "save_ai_provider"));
    }
}

/// A document window may only request its own saved document.
pub fn cloud_label(id: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("configuration-cloud-{:x}", Sha256::digest(id.as_bytes()))[..44].into()
}
pub fn cloud_scope(label: &str, command: &str, args: &serde_json::Value) -> bool {
    kind(label) != Some("cloud") || command == "get_settings" || args["id"].as_str().is_some_and(|id| cloud_label(id) == label)
}
#[cfg(test)] mod cloud_tests {
    use super::*;
    #[test] fn document_windows_cannot_cross_scope_or_send_chat() {
        let label=cloud_label("one");
        assert_eq!(kind(&label),Some("cloud"));
        assert!(command_allowed("cloud","library_cloud_read"));
        assert!(command_allowed("cloud","library_cloud_download"));
        assert!(!command_allowed("store","library_cloud_download"));
        assert!(!cloud_scope(&label,"library_cloud_download",&serde_json::json!({"id":"two"})));
        assert!(!command_allowed("cloud","chat_send"));
        assert!(!command_allowed("cloud","open_download_url"));
        assert!(cloud_scope(&label,"library_cloud_read",&serde_json::json!({"id":"one"})));
        assert!(!cloud_scope(&label,"library_cloud_read",&serde_json::json!({"id":"two"})));
    }
}
