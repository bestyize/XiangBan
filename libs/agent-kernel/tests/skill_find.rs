#![cfg(feature = "skills")]
mod common;
use agent_kernel::{skills::Skills, ToolCtx};
use common::TempDir;
use serde_json::{json, Value};
use std::sync::Arc;

#[test]
fn finds_installed_skills_outside_both_listing_limits_without_loading_bodies() {
    let root = TempDir::new("skill-find-overflow");
    for i in 0..220 {
        let name = format!("installed-{i:03}");
        root.write(&format!("{name}/SKILL.md"), &format!("---\nname: {name}\ndescription: installed capability {}\n---\nBODY_MUST_NOT_BE_SEARCHED_OR_RETURNED", "x".repeat(100)));
    }
    root.write("zz-calendar/SKILL.md", &format!("---\nname: zz-calendar\ndescription: installed 日历会议管理 {}\n---\nPRIVATE_BODY", "x".repeat(400)));
    let skills = Arc::new(Skills::new().read_only_root(&root.0));
    assert_eq!(skills.list().len(), 200);
    assert!(!skills.prompt_section()().contains("zz-calendar:"));
    let tools = skills.tools();
    let find = tools.iter().find(|t| t.spec().name == "skill_find").unwrap();
    let result: Value = serde_json::from_str(&find.execute(json!({"query":"日历"}), &ToolCtx::detached("s")).unwrap()).unwrap();
    assert_eq!(result["skills"][0]["name"], "zz-calendar");
    assert_eq!(result["total"], 1);
    assert!(!result.to_string().contains("PRIVATE_BODY"));
    let body: Value = serde_json::from_str(&find.execute(json!({"query":"BODY_MUST_NOT_BE_SEARCHED_OR_RETURNED"}), &ToolCtx::detached("s")).unwrap()).unwrap();
    assert_eq!(body["total"], 0);
    let last: Value = serde_json::from_str(&find.execute(json!({"query":"installed","page":12,"limit":20}), &ToolCtx::detached("s")).unwrap()).unwrap();
    assert_eq!(last["total"], 221);
    assert_eq!(last["skills"].as_array().unwrap().len(), 1);
    assert_eq!(last["hasMore"], false);
    assert!(skills.load("zz-calendar").unwrap().contains("PRIVATE_BODY"));
}

#[test]
fn sees_new_installs_and_keeps_root_priority_and_metadata_validation() {
    let first = TempDir::new("skill-find-first");
    let second = TempDir::new("skill-find-second");
    first.write("calendar/SKILL.md", "---\nname: calendar\ndescription: official 日历管理\n---\nbody");
    second.write("calendar/SKILL.md", "---\nname: calendar\ndescription: conflicting bogus provider\n---\nbody");
    second.write("broken/SKILL.md", "not valid metadata");
    let skills = Arc::new(Skills::new().read_only_root(&first.0).writable_root(&second.0));
    let tools = skills.tools();
    let find = tools.iter().find(|t| t.spec().name == "skill_find").unwrap();
    let ctx = ToolCtx::detached("s");
    let result = find.execute(json!({"query":"calendar"}), &ctx).unwrap();
    assert!(result.contains("official"));
    assert!(!result.contains("bogus"));
    second.write("new-skill/SKILL.md", "---\nname: new-skill\ndescription: freshlyinstalled\n---\nbody");
    let result: Value = serde_json::from_str(&find.execute(json!({"query":"freshlyinstalled"}), &ctx).unwrap()).unwrap();
    assert_eq!(result["total"], 1);
    let result: Value = serde_json::from_str(&find.execute(json!({"query":"broken"}), &ctx).unwrap()).unwrap();
    assert_eq!(result["total"], 0);
}
