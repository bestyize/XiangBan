//! 专家（WorkBuddy 的智能体人设）：对话绑定一个已安装的专家后，每次请求前把它的人设提示词放进系统消息。
//! 人设只改变做事的方法和交付标准：工具和权限仍由小伴统一管理，agent md 头部声明的 `tools` 一律忽略。
//! 提示词只加进这一次请求，不写进对话记录；专家被卸载后对话照常继续，只是不再有人设。
use agent_kernel::{HarnessError, Message, Middleware, Role, Turn};
use agent_skill_discovery::plugin::{self, AgentDoc};
use std::path::Path;

/// 人设正文最多放这么多字，避免一个超长的 agent md 占满上下文。
const MAX_PERSONA_CHARS: usize = 24_000;

/// 对话绑定的专家：装在哪个插件、用其中哪个 agent。名称和职业在绑定时记下，界面直接显示。
pub use super::sessions::ExpertBinding;

pub struct PersonaPrompt {
    text: String,
}

impl PersonaPrompt {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl Middleware for PersonaPrompt {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        // 紧跟在已有的系统消息后面，位置每次相同，利于提示词缓存。
        let at = turn.messages.iter().take_while(|m| m.role == Role::System).count();
        turn.messages.insert(at, Message::system(self.text.clone()));
        Ok(())
    }
}

pub struct Persona {
    pub text: String,
    pub max_turns: Option<u32>,
}

/// 人设提示词。`skills` 是 agent md 头部写的预装技能（技能本身已作为只读根加载，这里只提醒它们存在）。
pub fn persona_text(binding: &ExpertBinding, doc: &AgentDoc) -> String {
    let who = if binding.profession.is_empty() { binding.name.clone() } else { format!("{}，{}", binding.name, binding.profession) };
    let mut body: String = doc.body.trim().chars().take(MAX_PERSONA_CHARS).collect();
    if doc.body.trim().chars().count() > MAX_PERSONA_CHARS {
        body.push_str("\n…（人设过长，后面的内容已省略）");
    }
    let mut text = format!("## 当前专家\n这次对话由专家「{who}」负责，请按下面的人设和方法工作：\n\n{body}\n\n专家只改变做事的方法和交付标准，不增加本轮没有的工具、权限或记忆；需要什么能力仍按工具列表和权限提示来。");
    if !doc.skills.is_empty() {
        text.push_str(&format!("\n\n这位专家预装的技能：{}。需要时用 `load_skill` 加载后照着做。", doc.skills.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join("、")));
    }
    text
}

/// 读取对话绑定的专家。插件被卸载、agent 不存在或读不出来时返回 `None`。
pub fn load(plugins: &Path, binding: &ExpertBinding) -> Option<Persona> {
    let doc = plugin::read_agent(plugins, &binding.plugin, &binding.agent).ok()?;
    Some(Persona { text: persona_text(binding, &doc), max_turns: doc.max_turns })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn binding() -> ExpertBinding {
        ExpertBinding { plugin: "accounting-supervisor".into(), agent: "accounting-supervisor".into(), name: "AI 会计主管".into(), profession: "会计主管".into() }
    }

    fn doc(extra: &str) -> AgentDoc {
        plugin::parse_agent_md(&format!("---\nname: accounting-supervisor\ndescription: d\nmaxTurns: 30\n{extra}---\n# 会计主管\n\n你是会计主管，先问清业务实质。")).unwrap()
    }

    #[test]
    fn the_persona_text_names_the_expert_and_never_widens_what_the_run_may_do() {
        let text = persona_text(&binding(), &doc("skills: [ledger-check]\ntools: [Bash]\n"));
        assert!(text.contains("「AI 会计主管，会计主管」") && text.contains("你是会计主管，先问清业务实质"));
        assert!(text.contains("不增加本轮没有的工具、权限或记忆"));
        assert!(text.contains("`ledger-check`"));
        assert!(!text.contains("Bash"), "头部声明的 tools 被忽略，也不会进提示词");
        let long = AgentDoc { body: "字".repeat(MAX_PERSONA_CHARS + 50), ..doc("") };
        let cut = persona_text(&binding(), &long);
        assert!(cut.contains("人设过长") && cut.chars().count() < MAX_PERSONA_CHARS + 600);
    }

    #[test]
    fn the_prompt_goes_right_after_the_leading_system_messages_on_every_call() {
        let middleware = PersonaPrompt::new("PERSONA".into());
        let mut turn = Turn { session: "s".into(), index: 0, messages: vec![Message::system("sys"), Message::user_text("你好"), Message::assistant("好", vec![]), Message::user_text("继续")], tools: vec![] };
        middleware.before_model(&mut turn).unwrap();
        assert_eq!(turn.messages.len(), 5);
        assert_eq!((turn.messages[0].text(), turn.messages[1].text(), turn.messages[2].text()), ("sys".into(), "PERSONA".into(), "你好".into()));
        assert_eq!(turn.messages[1].role, Role::System);
        // 没有系统消息时放在最前面。
        let mut bare = Turn { session: "s".into(), index: 0, messages: vec![Message::user_text("你好")], tools: vec![] };
        middleware.before_model(&mut bare).unwrap();
        assert_eq!(bare.messages[0].text(), "PERSONA");
    }

    #[test]
    fn a_bound_expert_loads_from_the_installed_plugin_and_goes_quiet_when_it_is_gone() {
        let root = std::env::temp_dir().join(format!("persona-{}", std::process::id()));
        let backups = root.join("backups");
        let dir = root.join("plugins");
        let manifest = json!({"name": "accounting-supervisor", "version": "1.0.0", "description": "d", "agents": ["./agents/accounting-supervisor.md"], "expertType": "agent", "agentName": "accounting-supervisor"});
        let agent = "---\nname: accounting-supervisor\ndescription: d\nmaxTurns: 30\n---\n# 会计主管\n你是会计主管。";
        let zip = {
            use std::io::Write;
            let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            for (name, data) in [(".codebuddy-plugin/plugin.json", manifest.to_string()), ("agents/accounting-supervisor.md", agent.to_string())] {
                w.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
                w.write_all(data.as_bytes()).unwrap();
            }
            w.finish().unwrap().into_inner()
        };
        let pkg = plugin::inspect_zip(&zip).unwrap();
        plugin::install(&dir, &backups, &pkg, "local", json!({})).unwrap();
        let persona = load(&dir, &binding()).unwrap();
        assert_eq!(persona.max_turns, Some(30));
        assert!(persona.text.contains("你是会计主管。"));
        plugin::remove(&dir, "accounting-supervisor").unwrap();
        assert!(load(&dir, &binding()).is_none());
        assert!(load(&dir, &ExpertBinding { agent: "../x".into(), ..binding() }).is_none());
        let _ = fs::remove_dir_all(root);
    }
}
