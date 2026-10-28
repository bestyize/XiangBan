//! Only host-created configuration surfaces may request this narrow set of console operations.
pub fn kind(label: &str) -> Option<&'static str> {
    if label == "configuration-settings" { return Some("settings"); }
    let suffix = label.strip_prefix("configuration-model-")?;
    (suffix.len() == 24 && suffix.bytes().all(|c| c.is_ascii_hexdigit())).then_some("model")
}
pub fn command_allowed(role: &str, command: &str) -> bool {
    const COMMON: &[&str] = &[
        "get_settings", "get_app_info", "get_account_session", "get_website_status", "get_appearance",
        "get_ai_config", "refresh_ai_providers", "save_ai_provider", "delete_ai_provider", "switch_ai_provider",
        "save_ai_preferences", "fetch_ai_models", "pick_workspace", "open_download_url",
        "ai_gateway_balance", "ai_gateway_tiers", "ai_gateway_redeem", "use_ai_gateway",
    ];
    const SETTINGS: &[&str] = &[
        "get_startup_status", "get_game_status", "update_settings", "set_autostart", "open_data_directory",
        "set_appearance", "set_accent", "set_automatic_updates", "check_website", "dismiss_website_message",
        "get_network_config", "save_network_config", "test_network_connection", "assistant_status", "assistant_configure",
        "assistant_try", "open_screen_capture_settings", "reset_screen_capture_permission",
        "chat_agent_settings", "chat_set_command_sandbox", "chat_set_sandbox_network", "chat_set_execution_limits",
        "chat_set_windows_sandbox", "chat_save_hooks", "chat_revoke_grant",
        "tts_get_settings", "tts_save_settings", "tts_list_voices", "tts_synthesize", "tts_synthesize_part", "tts_cancel",
        "asr_get_status", "asr_set_enabled", "asr_prepare", "asr_report_microphone",
        "send_account_code", "login_account_with_code", "login_account_with_password", "register_account",
        "reset_account_password", "check_account_username", "get_signup_methods", "logout_account",
        "phone_link_status", "phone_link_set_enabled", "phone_link_begin_pairing", "phone_link_cancel_pairing", "phone_link_unpair",
        "speech_agent_env", "speech_agent_install_guides", "speech_agent_hooks_status", "speech_agent_hooks_install", "speech_agent_hooks_uninstall",
    ];
    matches!(role, "settings" | "model") && (COMMON.contains(&command) || role == "settings" && SETTINGS.contains(&command))
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn scope_is_closed_by_default() {
        assert_eq!(kind("configuration-settings"), Some("settings"));
        assert_eq!(kind("configuration-model-012345678901234567890123"), Some("model"));
        for label in ["wallpaper", "configuration-model-../control", "configuration-settings-other"] { assert!(kind(label).is_none()); }
        for role in ["settings", "model"] {
            assert!(!command_allowed(role, "chat_send"));
            assert!(!command_allowed(role, "library_remove"));
            assert!(!command_allowed(role, "configuration_window_reply"));
        }
        assert!(command_allowed("settings", "save_network_config"));
        assert!(!command_allowed("model", "save_network_config"));
        assert!(command_allowed("model", "save_ai_provider"));
    }
}
