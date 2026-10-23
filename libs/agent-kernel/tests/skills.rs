#![cfg(feature = "skills")]
mod common;

use agent_kernel::provider::scripted::Step;
use agent_kernel::skills::Skills;
use agent_kernel::*;
use common::*;
use std::sync::Arc;

fn skill_file(name: &str, description: &str, origin: Option<&str>, body: &str) -> String {
    let origin = origin.map(|o| format!("origin: {o}\n")).unwrap_or_default();
    format!("---\nname: {name}\ndescription: {description}\n{origin}---\n\n{body}\n")
}

fn manage(skills: &Arc<Skills>, args: &str) -> String {
    let steps = vec![Step::tool_call("c", "skill_manage", args), Step::text("ok")];
    let h = harness(steps, |mut b| {
        for t in skills.tools() {
            b = b.tool_arc(t);
        }
        b
    });
    assert!(run(&h.agent, "go").0.is_ok());
    tool_messages(&h).remove(0).1
}

fn setup() -> (TempDir, TempDir, Arc<Skills>) {
    let shared = TempDir::new("skills-shared");
    let user = TempDir::new("skills-user");
    let skills = Arc::new(Skills::new().read_only_root(&shared.0).writable_root(&user.0));
    (shared, user, skills)
}

#[test]
fn lists_valid_skills_and_ignores_malformed_ones() {
    let (shared, user, skills) = setup();
    shared.write("pdf-notes/SKILL.md", &skill_file("pdf-notes", "\"Summarise PDFs: carefully\"", None, "Read the PDF."));
    shared.write("crlf/SKILL.md", "\u{feff}---\r\nname: crlf\r\ndescription: windows file\r\n---\r\nbody\r\n");
    shared.write("wrong-name/SKILL.md", &skill_file("other-name", "mismatch", None, "x"));
    shared.write("no-description/SKILL.md", "---\nname: no-description\n---\nx\n");
    shared.write("no-frontmatter/SKILL.md", "just text\n");
    shared.write("Bad_Name/SKILL.md", &skill_file("Bad_Name", "uppercase", None, "x"));
    shared.write("stray-file.md", "not a skill");
    user.write("mine/SKILL.md", &skill_file("mine", "user skill", Some("agent"), "x"));
    let list = skills.list();
    let names: Vec<&str> = list.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["crlf", "mine", "pdf-notes"]);
    assert_eq!(list[2].description, "Summarise PDFs: carefully", "quotes are stripped, colons kept");
    assert_eq!((list[1].agent_created, list[1].writable), (true, true));
    assert_eq!((list[0].agent_created, list[0].writable), (false, false));
}

#[test]
fn earlier_roots_win_name_collisions() {
    let (shared, user, skills) = setup();
    shared.write("dup/SKILL.md", &skill_file("dup", "from shared", None, "shared body"));
    user.write("dup/SKILL.md", &skill_file("dup", "from user", Some("agent"), "user body"));
    assert_eq!(skills.find("dup").unwrap().description, "from shared");
    assert!(skills.load("dup").unwrap().contains("shared body"));
    assert_eq!(skills.list().len(), 1);
}

#[cfg(unix)]
#[test]
fn symlinked_skill_directories_are_ignored() {
    let (shared, _user, skills) = setup();
    let elsewhere = TempDir::new("skills-elsewhere");
    elsewhere.write("real/SKILL.md", &skill_file("linked", "via symlink", None, "x"));
    std::os::unix::fs::symlink(elsewhere.path("real"), shared.path("linked")).unwrap();
    assert!(skills.list().is_empty() && skills.find("linked").is_none());
}

#[test]
fn load_returns_the_body_without_frontmatter_and_names_alternatives_on_miss() {
    let (shared, _user, skills) = setup();
    shared.write("deploy/SKILL.md", &skill_file("deploy", "Deploy steps", None, "# Deploy\n1. Build\n2. Ship"));
    let loaded = skills.load("deploy").unwrap();
    assert!(loaded.starts_with("Skill `deploy` (directory: ") && loaded.ends_with("# Deploy\n1. Build\n2. Ship"));
    assert!(!loaded.contains("description:"));
    let miss = skills.load("deplyo").unwrap_err();
    assert!(miss.contains("no skill named `deplyo`") && miss.contains("deploy"));
    assert!(skills.load("../etc").is_err() && skills.find("a/b").is_none());
    shared.write("huge/SKILL.md", &skill_file("huge", "too big", None, &"x".repeat(200_000)));
    assert!(skills.load("huge").unwrap_err().contains("larger than"), "an oversized skill says why it cannot load");
    shared.write("broken/SKILL.md", "no frontmatter");
    assert!(skills.load("broken").unwrap_err().contains("frontmatter"));
}

#[test]
fn prompt_section_lists_skills_within_a_budget() {
    let (shared, _user, skills) = setup();
    let section = skills.prompt_section();
    assert_eq!(section(), "", "no section when there are no skills");
    shared.write("alpha/SKILL.md", &skill_file("alpha", "does alpha things", None, "x"));
    let text = section();
    assert!(text.starts_with("## Skills") && text.contains("- alpha: does alpha things") && text.contains("load_skill"));
    for i in 0..120 {
        shared.write(&format!("bulk-{i:03}/SKILL.md"), &skill_file(&format!("bulk-{i:03}"), &"d".repeat(200), None, "x"));
    }
    let text = section();
    assert!(text.len() < 8300 && text.contains("more skills not listed"), "{}", text.len());
}

#[test]
fn load_skill_tool_and_dynamic_prompt_work_inside_an_agent() {
    let (shared, _user, skills) = setup();
    shared.write("review/SKILL.md", &skill_file("review", "Review a diff", None, "Check tests first."));
    let steps = vec![Step::tool_call("c", "load_skill", r#"{"name":"review"}"#), Step::text("done")];
    let h = harness(steps, |mut b| {
        for t in skills.tools() {
            b = b.tool_arc(t);
        }
        b.system_prompt("You are helpful.").prompt_section(skills.prompt_section())
    });
    assert!(run(&h.agent, "go").0.is_ok());
    let system = h.provider.requests()[0].messages[0].text();
    assert!(system.starts_with("You are helpful.\n\n## Skills") && system.contains("- review: Review a diff"));
    assert!(tool_messages(&h)[0].1.contains("Check tests first."));
}

#[test]
fn a_skill_created_during_a_run_appears_in_the_next_model_call() {
    let (_shared, user, skills) = setup();
    let args = r#"{"action":"create","name":"release-notes","description":"Write release notes","content":"List changes by area."}"#;
    let h = harness(vec![Step::tool_call("c", "skill_manage", args), Step::text("saved")], |mut b| {
        for t in skills.tools() {
            b = b.tool_arc(t);
        }
        b.prompt_section(skills.prompt_section())
    });
    assert!(run(&h.agent, "go").0.is_ok());
    let requests = h.provider.requests();
    assert!(requests[0].messages.iter().all(|m| m.role != Role::System), "no skills yet, so no system message");
    assert!(requests[1].messages[0].text().contains("- release-notes: Write release notes"));
    let file = std::fs::read_to_string(user.path("release-notes/SKILL.md")).unwrap();
    // The description is written quoted, so any text is valid YAML.
    assert_eq!(file, "---\nname: release-notes\ndescription: \"Write release notes\"\norigin: agent\n---\n\nList changes by area.\n");
}

#[test]
fn skill_manage_validates_names_descriptions_and_sizes() {
    let (shared, user, skills) = setup();
    shared.write("taken/SKILL.md", &skill_file("taken", "already here", None, "x"));
    let create = |name: &str, desc: &str, content: &str| {
        manage(&skills, &serde_json::json!({"action": "create", "name": name, "description": desc, "content": content}).to_string())
    };
    for bad in ["../escape", "A", "-lead", "trail-", "has space", "", "a/b", &"x".repeat(65)] {
        assert!(create(bad, "d", "c").contains("`name` must be"), "name {bad:?}");
    }
    assert!(create("ok", "line one\nline two", "c").contains("one line"));
    assert!(create("ok", "   ", "c").contains("one line"));
    assert!(create("ok", &"d".repeat(501), "c").contains("one line"));
    assert!(create("ok", "d", "  ").contains("`content` must be"));
    assert!(create("ok", "d", &"c".repeat(20_001)).contains("`content` must be"));
    assert!(create("taken", "d", "c").contains("already exists"));
    assert!(manage(&skills, r#"{"action":"explode","name":"ok"}"#).contains("/action"));
    assert!(!user.path("ok").exists() && !user.path("taken").exists() && !user.path("../escape").exists());
}

#[test]
fn only_agent_created_skills_in_the_writable_root_can_be_changed() {
    let (shared, user, skills) = setup();
    shared.write("bundled/SKILL.md", &skill_file("bundled", "shipped", None, "keep me"));
    user.write("handmade/SKILL.md", &skill_file("handmade", "written by the user", None, "keep me too"));
    user.write("agentic/SKILL.md", &skill_file("agentic", "made by the assistant", Some("agent"), "step one\nstep two"));
    for name in ["bundled", "handmade"] {
        for action in [r#"{"action":"patch","old_string":"keep","new_string":"x"}"#, r#"{"action":"delete"}"#] {
            let args = action.replacen('{', &format!(r#"{{"name":"{name}","#), 1);
            assert!(manage(&skills, &args).contains("was not created by the assistant"), "{name} {action}");
        }
    }
    assert!(skills.load("bundled").unwrap().contains("keep me") && skills.load("handmade").unwrap().contains("keep me too"));

    assert_eq!(manage(&skills, r#"{"action":"patch","name":"agentic","old_string":"step two","new_string":"step 2"}"#), "Updated skill `agentic`.");
    assert!(skills.load("agentic").unwrap().ends_with("step one\nstep 2"));
    assert!(std::fs::read_to_string(user.path("agentic/SKILL.md")).unwrap().contains("origin: agent"), "header is preserved");
    assert!(manage(&skills, r#"{"action":"patch","name":"agentic","old_string":"step","new_string":"s"}"#).contains("occurs 2 times"));
    assert!(manage(&skills, r#"{"action":"patch","name":"agentic","old_string":"missing","new_string":"s"}"#).contains("not found"));
    assert!(manage(&skills, r#"{"action":"patch","name":"agentic","old_string":"","new_string":"s"}"#).contains("must not be empty"));
    let big = serde_json::json!({"action": "patch", "name": "agentic", "old_string": "step one", "new_string": "x".repeat(21_000)});
    assert!(manage(&skills, &big.to_string()).contains("would exceed"));
}

#[test]
fn patch_and_delete_keep_a_backup_and_delete_spares_foreign_files() {
    let (_shared, user, skills) = setup();
    user.write("tmp-skill/SKILL.md", &skill_file("tmp-skill", "temporary", Some("agent"), "original body"));
    user.write("tmp-skill/notes.txt", "user's own file");
    assert!(manage(&skills, r#"{"action":"patch","name":"tmp-skill","old_string":"original","new_string":"edited"}"#).starts_with("Updated"));
    let backups = |prefix: &str| std::fs::read_dir(user.path(".history")).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with(prefix)).count();
    assert_eq!(backups("tmp-skill."), 1);
    let old = std::fs::read_dir(user.path(".history")).unwrap().next().unwrap().unwrap().path();
    assert!(std::fs::read_to_string(old).unwrap().contains("original body"));

    let out = manage(&skills, r#"{"action":"delete","name":"tmp-skill"}"#);
    assert!(out.starts_with("Deleted skill `tmp-skill`.") && out.contains("left in place"), "{out}");
    assert!(!user.path("tmp-skill/SKILL.md").exists() && user.path("tmp-skill/notes.txt").exists());
    assert_eq!(backups("tmp-skill."), 2);
    assert!(skills.find("tmp-skill").is_none());

    user.write("clean/SKILL.md", &skill_file("clean", "no extras", Some("agent"), "x"));
    assert_eq!(manage(&skills, r#"{"action":"delete","name":"clean"}"#), "Deleted skill `clean`.");
    assert!(!user.path("clean").exists());
}

#[test]
fn skill_manage_is_only_offered_with_a_writable_root() {
    let dir = TempDir::new("skills-ro");
    let read_only = Arc::new(Skills::new().read_only_root(&dir.0));
    let names: Vec<String> = read_only.tools().iter().map(|t| t.spec().name.clone()).collect();
    assert_eq!(names, vec!["load_skill", "skill_find"]);
    let writable = Arc::new(Skills::new().writable_root(&dir.0));
    let names: Vec<String> = writable.tools().iter().map(|t| t.spec().name.clone()).collect();
    assert_eq!(names, vec!["load_skill", "skill_find", "skill_manage"]);
}

#[test]
fn skill_manage_needs_approval_even_in_auto_mode() {
    let (_shared, user, skills) = setup();
    let permission = Arc::new(Permission::new(PermissionMode::Auto, |_: &ApprovalRequest<'_>| Approval::Deny));
    let args = r#"{"action":"create","name":"sneaky","description":"d","content":"c"}"#;
    let h = harness(vec![Step::tool_call("c", "skill_manage", args), Step::text("ok")], |mut b| {
        for t in skills.tools() {
            b = b.tool_arc(t);
        }
        b.middleware(permission.clone())
    });
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(tool_messages(&h)[0].1.contains("denied permission"));
    assert!(!user.path("sneaky").exists());
}
