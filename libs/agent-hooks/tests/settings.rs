use std::{fs, path::PathBuf};
use xiaoban_agent_hooks::claude_settings::{self as cs, InstallSpec, Scope};
fn spec() -> InstallSpec {
    let mut s = InstallSpec::new(
        std::env::current_dir()
            .unwrap()
            .join("含 空格/xiaoban-hook.exe"),
    );
    s.events = vec!["Stop".into()];
    s
}
#[test]
fn coexist_idempotent_restore_and_status() {
    let dir = tempfile::tempdir().unwrap();
    let scope = Scope::Project(dir.path().into());
    let path = dir.path().join(".claude/settings.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let original="{\n    \"z\": 1,\n    \"hooks\": {\n        \"Stop\": [{\"matcher\": \"x\", \"hooks\": [{\"type\": \"command\", \"command\": \"other\"}]}]\n    },\n    \"a\": 2\n}\n";
    fs::write(&path, original).unwrap();
    let plan = cs::plan_install(&scope, &spec()).unwrap();
    assert!(plan.after.find("\"z\"").unwrap() < plan.after.find("\"hooks\"").unwrap());
    assert!(plan.after.find("other").unwrap() < plan.after.find("xiaoban-hook").unwrap());
    assert!(cs::apply(&plan).unwrap().backup.unwrap().exists());
    assert!(cs::status(&scope).unwrap().other_hooks);
    assert!(!cs::status(&scope).unwrap().installed[0].2);
    let again = cs::plan_install(&scope, &spec()).unwrap();
    assert_eq!(again.before, again.after);
    let remove = cs::plan_uninstall(&scope).unwrap();
    cs::apply(&remove).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
}
#[test]
fn absent_invalid_stale_backup_and_restore() {
    let dir = tempfile::tempdir().unwrap();
    let scope = Scope::ProjectLocal(dir.path().into());
    let path = dir.path().join(".claude/settings.local.json");
    let plan = cs::plan_install(&scope, &spec()).unwrap();
    cs::apply(&plan).unwrap();
    assert!(path.exists());
    let plan = cs::plan_install(&scope, &spec()).unwrap();
    fs::write(&path, "{}\n").unwrap();
    assert!(cs::apply(&plan).is_err());
    fs::write(&path, "{ // comment\n}\n").unwrap();
    assert!(cs::plan_install(&scope, &spec()).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "{ // comment\n}\n");
    fs::write(&path, "{}\n").unwrap();
    for i in 0..7 {
        let mut s = spec();
        s.forwarder = PathBuf::from(format!("{}/版本{i}/xiaoban-hook.exe", dir.path().display()));
        let p = cs::plan_install(&scope, &s).unwrap();
        cs::apply(&p).unwrap();
    }
    let count = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains("xiaoban-backup-"))
        .count();
    assert_eq!(count, 5);
    cs::restore_latest_backup(&scope).unwrap();
    assert!(fs::read_to_string(path).unwrap().contains("版本5"));
}
#[test]
fn retain_preexisting_empty_groups() {
    let d = tempfile::tempdir().unwrap();
    let scope = Scope::Project(d.path().into());
    let path = d.path().join(".claude/settings.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let original = "{\"hooks\":{\"Stop\":[{\"hooks\":[]}]}}";
    fs::write(&path, original).unwrap();
    cs::apply(&cs::plan_install(&scope, &spec()).unwrap()).unwrap();
    cs::apply(&cs::plan_uninstall(&scope).unwrap()).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), original);
}

#[test]
fn config_dir_and_coexistence() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("配置");
    fs::create_dir_all(&config).unwrap();
    let old_config = std::env::var_os("CLAUDE_CONFIG_DIR");
    let old_home = std::env::var_os("HOME");
    std::env::set_var("CLAUDE_CONFIG_DIR", &config);
    std::env::set_var("HOME", dir.path());
    fs::write(
        dir.path().join(".claude.json"),
        r#"{"mcpServers":{"mcp-voice-hooks":{"command":"voice"},"voicemode":{"command":"mode"}}}"#,
    )
    .unwrap();
    let settings = config.join("settings.json");
    fs::write(&settings, r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"mcp-voice-hooks"},{"type":"command","command":"voicemode"}]}]}}"#).unwrap();
    let detected = cs::detect_coexisting();
    assert!(detected.iter().any(|c| c.name == "mcp-voice-hooks"));
    assert!(detected.iter().any(|c| c.name == "voicemode"));
    let plan = cs::plan_install(&Scope::User, &spec()).unwrap();
    assert_eq!(plan.path, settings);
    let mcp_file = dir.path().join(".claude.json");
    assert!(detected
        .iter()
        .any(|c| c.name == "mcp-voice-hooks" && c.source == mcp_file));
    assert!(detected
        .iter()
        .any(|c| c.name == "voicemode" && c.source == mcp_file));
    if let Some(v) = old_config {
        std::env::set_var("CLAUDE_CONFIG_DIR", v);
    } else {
        std::env::remove_var("CLAUDE_CONFIG_DIR");
    }
    if let Some(v) = old_home {
        std::env::set_var("HOME", v);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn indent_order_and_unknown_structures_survive() {
    for indent in ["  ", "    ", "\t"] {
        let d = tempfile::tempdir().unwrap();
        let scope = Scope::Project(d.path().into());
        let path = d.path().join(".claude/settings.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let original = format!("{{\n{indent}\"first\": 1,\n{indent}\"hooks\": {{\n{indent}{indent}\"opaque\": {{\"untouched\": true}},\n{indent}{indent}\"Stop\": [{{\"matcher\": \"x\", \"hooks\": [{{\"type\": \"http\", \"url\": \"other\"}}]}}]\n{indent}}},\n{indent}\"last\": 2\n}}\n");
        fs::write(&path, &original).unwrap();
        let plan = cs::plan_install(&scope, &spec()).unwrap();
        assert!(plan.after.contains(&format!("\n{indent}\"first\"")));
        cs::apply(&plan).unwrap();
        let mut current: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            current["hooks"]["opaque"],
            serde_json::json!({"untouched": true})
        );
        current["external"] = serde_json::json!("changed after installation");
        fs::write(&path, serde_json::to_vec_pretty(&current).unwrap()).unwrap();
        cs::apply(&cs::plan_uninstall(&scope).unwrap()).unwrap();
        let result: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(result["hooks"]["Stop"][0]["hooks"][0]["url"], "other");
        assert_eq!(
            result["hooks"]["opaque"],
            serde_json::json!({"untouched": true})
        );
        assert_eq!(result["external"], "changed after installation");
        assert_eq!(result["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }
}
#[test]
fn default_events_path_update_and_missing_executable() {
    let d = tempfile::tempdir().unwrap();
    let scope = Scope::Project(d.path().into());
    let old = d.path().join("路径 with space").join(if cfg!(windows) {
        "xiaoban-hook.exe"
    } else {
        "xiaoban-hook"
    });
    fs::create_dir_all(old.parent().unwrap()).unwrap();
    fs::write(&old, b"test").unwrap();
    let mut spec = InstallSpec::new(old.clone());
    let plan = cs::plan_install(&scope, &spec).unwrap();
    let root: serde_json::Value = serde_json::from_str(&plan.after).unwrap();
    assert_eq!(root["hooks"].as_object().unwrap().len(), 8);
    assert_eq!(
        root["hooks"]["PermissionRequest"][0]["hooks"][0]["timeout"],
        130
    );
    assert_eq!(root["hooks"]["Stop"][0]["hooks"][0]["timeout"], 30);
    assert_eq!(
        root["hooks"]["Stop"][0]["hooks"][0]["args"],
        serde_json::json!(["Stop"])
    );
    cs::apply(&plan).unwrap();
    assert_eq!(cs::status(&scope).unwrap().installed.len(), 8);
    assert!(cs::status(&scope).unwrap().installed.iter().all(|x| x.2));
    fs::remove_file(&old).unwrap();
    assert!(cs::status(&scope).unwrap().installed.iter().all(|x| !x.2));
    spec.forwarder = d.path().join("new path").join(if cfg!(windows) {
        "xiaoban-hook.exe"
    } else {
        "xiaoban-hook"
    });
    cs::apply(&cs::plan_install(&scope, &spec).unwrap()).unwrap();
    assert!(cs::status(&scope)
        .unwrap()
        .installed
        .iter()
        .all(|x| x.1 == spec.forwarder));
    cs::apply(&cs::plan_uninstall(&scope).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &fs::read(d.path().join(".claude/settings.json")).unwrap()
        )
        .unwrap(),
        serde_json::json!({})
    );
}
#[test]
fn invalid_json_never_changes_existing_file() {
    for text in ["{ broken", "{\"x\":1,}", "{ // JSONC\n}", "[]"] {
        let d = tempfile::tempdir().unwrap();
        let scope = Scope::Project(d.path().into());
        let path = d.path().join(".claude/settings.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        assert!(cs::plan_install(&scope, &spec()).is_err());
        assert!(cs::plan_uninstall(&scope).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), text);
    }
}
