//! 用户用 `/技能名` 调用技能（WorkBuddy 的做法）：请求前，把用户最新消息里点名的技能的完整说明放到这条消息后面。
//! 和 `feature_hints` 一样只加进这一次请求，不写进对话记录；技能只提供做法，不增加权限。
//! 标记了 `disable-model-invocation` 的技能只能这样启动，模型自己不会在索引里看到它们；
//! 标记了 `user-invocable: false` 的技能不会被这条路径加载。
use agent_kernel::skills::Skills;
use agent_kernel::{HarnessError, Message, Middleware, Role, Turn};
use serde_json::{json, Value};
use std::{path::Path, sync::Arc};

/// 一条消息里最多展开的技能数，避免一条消息塞进一堆说明。
const MAX_INVOKED: usize = 3;

/// 消息里按 `/名称` 点名的技能名，按出现顺序、去重。名称必须紧跟在行首或空白后面，并且后面不能接
/// 路径或文件名的字符（`/etc/hosts`、`/docs.md` 都不是调用）。
pub fn invoked(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut names: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let starts = chars[i] == '/' && (i == 0 || chars[i - 1].is_whitespace());
        if !starts {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < chars.len() && (chars[j].is_ascii_lowercase() || chars[j].is_ascii_digit() || chars[j] == '-') {
            j += 1;
        }
        let ends_cleanly = chars.get(j).is_none_or(|c| !(c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '\\' | ':')));
        if j > i + 1 && ends_cleanly {
            let name: String = chars[i + 1..j].iter().collect();
            if !name.starts_with('-') && !name.ends_with('-') && name.len() <= 64 && !names.contains(&name) {
                names.push(name);
            }
        }
        i = j.max(i + 1);
    }
    names
}

pub struct SkillInvocation {
    skills: Arc<Skills>,
}

impl SkillInvocation {
    pub fn new(skills: Arc<Skills>) -> Self {
        Self { skills }
    }
}

impl Middleware for SkillInvocation {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        // Right after the user's own message, so a tool call keeps its result next to it.
        let Some(at) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()) };
        let text = turn.messages[at].text();
        let loaded: Vec<(String, String)> = invoked(&text)
            .into_iter()
            .filter_map(|name| self.skills.load_for_user(&name).ok().map(|body| (name, body)))
            .take(MAX_INVOKED)
            .collect();
        if loaded.is_empty() {
            return Ok(());
        }
        let mut reminder = String::new();
        for (name, body) in &loaded {
            reminder.push_str(&format!(
                "用户在这条消息里用 /{name} 调用了技能「{name}」。下面是它的完整说明，请照着完成用户的请求；技能只提供做法，不增加本轮没有的工具、权限或记忆。\n\n{body}\n\n"
            ));
        }
        turn.messages.insert(at + 1, Message::reminder(reminder.trim_end().to_string()));
        Ok(())
    }
}

/// `/` 菜单里的一项。`plugins` / `skills` 是插件目录和用户技能目录，用来判断来源。
pub fn menu(skills: &Skills, plugins: &Path, user_skills: &Path) -> Vec<Value> {
    skills
        .invocable()
        .into_iter()
        .map(|s| {
            let (source, plugin) = match s.dir.strip_prefix(plugins).ok().and_then(|rest| rest.components().next()) {
                Some(first) => ("plugin", Some(first.as_os_str().to_string_lossy().into_owned())),
                None if s.dir.starts_with(user_skills) => (if s.agent_created { "assistant" } else { "installed" }, None),
                None => ("builtin", None),
            };
            json!({
                "name": s.name,
                "displayName": s.display_name.clone().unwrap_or_else(|| s.name.clone()),
                "description": s.description_zh.clone().unwrap_or_else(|| s.description.clone()),
                "descriptionEn": s.description_en,
                "source": source, "plugin": plugin,
                "version": s.version, "author": s.author, "category": s.category, "tags": s.tags,
                "modelInvocable": s.model_invocable, "allowedTools": s.allowed_tools,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn only_a_whole_skill_name_after_whitespace_is_an_invocation() {
        assert_eq!(invoked("/pptx 做一份季度汇报"), ["pptx"]);
        assert_eq!(invoked("先 /xlsx 再 /pptx，最后 /xlsx"), ["xlsx", "pptx"], "按出现顺序、去重，中文逗号后能结束名称");
        assert_eq!(invoked("看下 /etc/hosts 和 /docs.md、 C:/x、a/b、/Users"), Vec::<String>::new(), "路径和文件名不是调用");
        assert_eq!(invoked("https://x.com/pptx 和 //pptx 和 /-a 和 /a-"), Vec::<String>::new());
        assert_eq!(invoked("/a\n/b"), ["a", "b"]);
        assert!(invoked("").is_empty() && invoked("/").is_empty());
    }

    fn skills_in(root: &Path) -> Arc<Skills> {
        for (name, flags) in [("pptx", ""), ("user-only", "disable-model-invocation: true\n"), ("model-only", "user-invocable: false\n")] {
            let dir = root.join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), format!("---\nname: {name}\ndescription: the {name} skill\n{flags}---\nSteps of {name}")).unwrap();
        }
        Arc::new(Skills::new().read_only_root(root))
    }

    fn turn(messages: Vec<Message>) -> Turn {
        Turn { session: "s".into(), index: 0, messages, tools: vec![] }
    }

    #[test]
    fn an_invoked_skill_lands_right_after_the_request_and_only_for_that_request() {
        let root = std::env::temp_dir().join(format!("skill-invocation-{}", std::process::id()));
        let middleware = SkillInvocation::new(skills_in(&root));
        let mut matching = turn(vec![Message::system("sys"), Message::user_text("旧问题"), Message::assistant("旧回答", vec![]), Message::user_text("/pptx 做一份汇报")]);
        middleware.before_model(&mut matching).unwrap();
        assert_eq!(matching.messages.len(), 5);
        assert!(matching.messages[4].synthetic && matching.messages[4].text().contains("Steps of pptx"));
        assert_eq!(matching.messages[3].text(), "/pptx 做一份汇报", "记录里的用户消息保持原样");
        // 用户只能调用允许用户调用的技能，模型专用的不会被这条路径加载；点名不存在的技能什么也不加。
        let mut hand_started = turn(vec![Message::user_text("/user-only 来")]);
        middleware.before_model(&mut hand_started).unwrap();
        assert!(hand_started.messages[1].text().contains("Steps of user-only"));
        let mut not_for_hands = turn(vec![Message::user_text("/model-only /nothing-here 来")]);
        middleware.before_model(&mut not_for_hands).unwrap();
        assert_eq!(not_for_hands.messages.len(), 1);
        // 之后的步骤里说明仍在用户请求旁边，工具调用在它后面。
        let mut stepped = turn(vec![Message::user_text("/pptx 开始"), Message::assistant("", vec![])]);
        middleware.before_model(&mut stepped).unwrap();
        assert!(stepped.messages[1].synthetic && !stepped.messages[2].synthetic);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_menu_lists_what_the_user_may_start_and_where_each_comes_from() {
        let base = std::env::temp_dir().join(format!("skill-menu-{}", std::process::id()));
        let (plugins, user, bundled) = (base.join("plugins"), base.join("skills"), base.join("bundled"));
        for (root, name) in [(&plugins.join("acme/skills"), "from-plugin"), (&user, "mine"), (&bundled, "shipped")] {
            let dir = root.join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), format!("---\nname: {name}\ndescription: d\ndisplay_name: \"中文名\"\ndescription_zh: 中文说明\nversion: 2.0\n---\nB")).unwrap();
        }
        let skills = Skills::new().read_only_root(&bundled).writable_root(&user).read_only_root(plugins.join("acme/skills"));
        let items = menu(&skills, &plugins, &user);
        let by = |n: &str| items.iter().find(|i| i["name"] == n).unwrap().clone();
        assert_eq!((by("from-plugin")["source"].as_str(), by("from-plugin")["plugin"].as_str()), (Some("plugin"), Some("acme")));
        assert_eq!((by("mine")["source"].as_str(), by("shipped")["source"].as_str()), (Some("installed"), Some("builtin")));
        assert_eq!((by("mine")["displayName"].as_str(), by("mine")["description"].as_str(), by("mine")["version"].as_str()), (Some("中文名"), Some("中文说明"), Some("2.0")));
        fs::remove_dir_all(base).unwrap();
    }
}
