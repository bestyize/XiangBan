//! 把 SkillHub 的专家包（一份串联若干技能的工作流）转成 WorkBuddy 格式的专家插件：
//! 工作流正文成为人设的“工作流程”，它依赖的子技能打进插件的 `skills/`，装上专家就能直接用，不再缺依赖。
//! SkillHub 没有头像、标签和推荐提问，这些按名称和场景生成，并在条目里标记 `generated`，由管理员在后台改。
use crate::plugin::CATEGORIES;
use serde_json::{json, Value};
use std::{collections::BTreeMap, io::Write};

/// 一个子技能：路径相对于技能目录（`SKILL.md`、`scripts/run.py`…），已经过包校验。
pub struct Child {
    pub slug: String,
    pub files: BTreeMap<String, Vec<u8>>,
}

pub struct Source<'a> {
    pub slug: &'a str,
    pub name_zh: &'a str,
    pub name_en: &'a str,
    pub summary: &'a str,
    pub summary_en: &'a str,
    pub content: &'a str,
    pub category_id: &'a str,
    pub version: &'a str,
}

/// 去掉 SKILL.md 开头的 `---` 头部，留下正文。
pub fn strip_header(md: &str) -> &str {
    let text = md.strip_prefix('\u{feff}').unwrap_or(md);
    let Some(rest) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")) else { return text };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        offset += line.len();
        if line.trim_end() == "---" {
            return rest[offset..].trim_start_matches(['\r', '\n']);
        }
    }
    text
}

fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate(s: &str, max: usize) -> String {
    let s = one_line(s);
    if s.chars().count() <= max {
        s
    } else {
        s.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
    }
}

/// YAML 双引号字符串。
fn q(s: &str) -> String {
    serde_json::to_string(&one_line(s)).unwrap_or_else(|_| "\"\"".into())
}

fn category_names(id: &str) -> (&'static str, &'static str) {
    CATEGORIES.iter().find(|(c, _, _)| *c == id).map(|(_, zh, en)| (*zh, *en)).unwrap_or(("行业顾问", "Industry Consultant"))
}

/// plugin.json 的内容。
pub fn manifest(src: &Source, skills: &[String]) -> Value {
    let name_en = if src.name_en.trim().is_empty() { src.slug } else { src.name_en };
    let (cat_zh, cat_en) = category_names(src.category_id);
    let first = json!({ "zh": format!("请用「{}」帮我规划这项工作的整体方案", src.name_zh), "en": format!("Plan the whole job with {name_en}") });
    let mut manifest = json!({
        "name": src.slug, "version": src.version,
        "description": truncate(if src.summary_en.is_empty() { src.summary } else { src.summary_en }, 300),
        "author": { "name": "SkillHub" },
        "agents": [format!("./agents/{}.md", src.slug)],
        "expertType": "agent", "agentName": src.slug,
        "displayName": { "en": name_en, "zh": src.name_zh },
        "profession": { "en": name_en, "zh": src.name_zh },
        "displayDescription": { "en": truncate(if src.summary_en.is_empty() { src.summary } else { src.summary_en }, 160), "zh": truncate(src.summary, 50) },
        "categoryId": src.category_id,
        "defaultInitPrompt": first,
        "plugin": src.slug,
        "tags": [
            { "zh": cat_zh, "en": cat_en },
            { "zh": truncate(src.name_zh, 12), "en": truncate(name_en, 24) },
            { "zh": "SkillHub", "en": "SkillHub" },
        ],
        "quickPrompts": [
            first,
            { "zh": format!("按「{}」的流程，一步步带我完成任务", src.name_zh), "en": format!("Walk me through the task step by step with {name_en}") },
            { "zh": "我有一个相关任务，请先问我几个关键问题再开始", "en": "I have a task in this area - ask me the key questions first" },
        ],
    });
    if !skills.is_empty() {
        manifest["skills"] = json!(skills.iter().map(|s| format!("./skills/{s}")).collect::<Vec<_>>());
    }
    manifest
}

/// agent md：头部是 WorkBuddy 规定的字段，正文是人设加工作流程。
pub fn agent_md(src: &Source, skills: &[String]) -> String {
    let name_en = if src.name_en.trim().is_empty() { src.slug } else { src.name_en };
    let mut body = format!(
        "# {zh}\n\n你是「{zh}」专家。{summary}\n\n按下面的工作流程帮用户完成任务，每一步先说清楚在做什么，产出要具体、可直接使用。",
        zh = src.name_zh, summary = one_line(src.summary),
    );
    if !skills.is_empty() {
        body.push_str(&format!("\n\n流程里用到的技能已随本专家安装：{}。需要时用 `load_skill` 加载对应的技能再照着做。", skills.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join("、")));
    }
    body.push_str("\n\n## 工作流程\n\n");
    body.push_str(strip_header(src.content).trim());
    body.push('\n');
    format!(
        "---\nname: {slug}\ndescription: {desc}\ndisplayName:\n  en: {en}\n  zh: {zh}\nprofession:\n  en: {en}\n  zh: {zh}\nmaxTurns: 50\n---\n\n{body}",
        slug = src.slug, desc = q(if src.summary_en.is_empty() { src.summary } else { src.summary_en }), en = q(name_en), zh = q(src.name_zh),
    )
}

fn readme(src: &Source, skills: &[String], missing: &[String]) -> String {
    let mut text = format!("# {}\n\n{}\n\n来源：SkillHub 专家包 `{}`，已转换为 WorkBuddy 插件格式。\n", src.name_zh, one_line(src.summary), src.slug);
    if !skills.is_empty() {
        text.push_str(&format!("\n## 随包安装的技能\n\n{}\n", skills.iter().map(|s| format!("- {s}")).collect::<Vec<_>>().join("\n")));
    }
    if !missing.is_empty() {
        text.push_str(&format!("\n## 暂时没有带上的技能\n\n{}\n", missing.iter().map(|s| format!("- {s}")).collect::<Vec<_>>().join("\n")));
    }
    text
}

/// 组装插件 ZIP。同样的输入得到同样的字节（固定时间戳、固定顺序），所以重复入库不会产生新版本。
pub fn skillhub_expert(src: &Source, children: &[Child], missing: &[String]) -> Result<Vec<u8>, String> {
    let skills: Vec<String> = children.iter().map(|c| c.slug.clone()).collect();
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    entries.insert(".codebuddy-plugin/plugin.json".into(), serde_json::to_vec_pretty(&manifest(src, &skills)).map_err(|e| e.to_string())?);
    entries.insert(format!("agents/{}.md", src.slug), agent_md(src, &skills).into_bytes());
    entries.insert("README.md".into(), readme(src, &skills, missing).into_bytes());
    for child in children {
        for (path, bytes) in &child.files {
            entries.insert(format!("skills/{}/{path}", child.slug), bytes.clone());
        }
    }
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).last_modified_time(zip::DateTime::default());
    for (path, bytes) in &entries {
        writer.start_file(path.as_str(), opts).map_err(|e| format!("打包失败：{e}"))?;
        writer.write_all(bytes).map_err(|e| format!("打包失败：{e}"))?;
    }
    writer.finish().map(|c| c.into_inner()).map_err(|e| format!("打包失败：{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin;

    const CONTENT: &str = "---\nname: tech-test-automation\ndescription: x\nversion: 1.0.0\n---\n# 自动化测试工作流\n\n## 步骤 1\n使用 **superpowers-tdd**。\n";

    fn source<'a>() -> Source<'a> {
        Source { slug: "tech-test-automation", name_zh: "自动化测试", name_en: "Test Automation", summary: "从 TDD 方法论到自动生成单元测试代码、E2E 编排、API 测试自动化，再到 QA 测试计划与覆盖率矩阵生成的完整自动化测试工作流。", summary_en: "", content: CONTENT, category_id: "02-Engineering", version: "1.0.0" }
    }

    fn child(slug: &str) -> Child {
        Child { slug: slug.into(), files: BTreeMap::from([("SKILL.md".to_string(), format!("---\nname: {slug}\ndescription: d\n---\nbody").into_bytes()), ("scripts/run.py".to_string(), b"print(1)".to_vec())]) }
    }

    #[test]
    fn the_header_of_a_workflow_is_dropped_and_the_body_kept() {
        assert!(strip_header(CONTENT).starts_with("# 自动化测试工作流"));
        assert_eq!(strip_header("no header"), "no header");
        assert_eq!(strip_header("---\nname: x\nbody without closing"), "---\nname: x\nbody without closing");
    }

    #[test]
    fn a_converted_expert_is_a_valid_workbuddy_plugin_with_its_skills_inside() {
        let zip = skillhub_expert(&source(), &[child("superpowers-tdd"), child("test-patterns")], &["gone-skill".into()]).unwrap();
        let pkg = plugin::inspect_plugin_zip(&zip).unwrap();
        assert_eq!((pkg.manifest.name.as_str(), pkg.manifest.expert_type.as_str(), pkg.manifest.category_id.as_str()), ("tech-test-automation", "agent", "02-Engineering"));
        assert_eq!(pkg.manifest.skills, ["./skills/superpowers-tdd", "./skills/test-patterns"]);
        assert!(pkg.unpacked.contents.contains_key("skills/superpowers-tdd/scripts/run.py"));
        let doc = &pkg.agents["agents/tech-test-automation.md"];
        assert_eq!((doc.name.as_str(), doc.max_turns), ("tech-test-automation", Some(50)));
        assert_eq!(doc.profession.as_ref().unwrap().zh, "自动化测试");
        assert!(doc.body.contains("`superpowers-tdd`") && doc.body.contains("## 工作流程") && doc.body.contains("使用 **superpowers-tdd**"));
        assert!(!doc.body.contains("version: 1.0.0"), "工作流的头部不进正文");
        // 缺失的技能写进说明；没有头像，所以只有宽松模式下才通过，且只有这一类警告。
        assert!(String::from_utf8_lossy(&pkg.unpacked.contents["README.md"]).contains("gone-skill"));
        let issues = plugin::validate_expert(&pkg.manifest, &pkg.unpacked.contents, true);
        assert!(issues.iter().filter(|i| i.error).all(|i| i.message.contains("头像")), "{issues:?}");
    }

    #[test]
    fn the_same_input_gives_the_same_bytes_so_reimporting_does_not_make_new_versions() {
        let a = skillhub_expert(&source(), &[child("a-skill")], &[]).unwrap();
        let b = skillhub_expert(&source(), &[child("a-skill")], &[]).unwrap();
        assert_eq!(a, b);
        let changed = skillhub_expert(&Source { content: "# 变了", ..source() }, &[child("a-skill")], &[]).unwrap();
        assert_ne!(a, changed);
    }

    #[test]
    fn generated_presentation_fits_workbuddys_shape() {
        let m = manifest(&source(), &[]);
        assert_eq!(m["tags"].as_array().unwrap().len(), 3);
        assert_eq!(m["quickPrompts"].as_array().unwrap().len(), 3);
        assert_eq!(m["defaultInitPrompt"], m["quickPrompts"][0]);
        assert!(m["displayDescription"]["zh"].as_str().unwrap().chars().count() <= 50);
        assert!(m.get("skills").is_none(), "没有子技能就不写 skills");
        assert_eq!(m["tags"][0]["zh"], "技术工程");
    }
}
