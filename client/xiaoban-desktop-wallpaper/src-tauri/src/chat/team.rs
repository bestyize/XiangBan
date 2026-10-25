//! 专家团（WorkBuddy 的 `expertType: team`）：对话绑定一个专家团后，主理人拿到 TeamCreate / Agent / SendMessage /
//! TeamDelete 四个工具，按它自己的 SOP 把任务分给成员。每个成员是一个独立的 agent：人设来自它在插件里的 agent md，
//! 工具和权限与主对话共用（同一个 `Permission`，确认卡片出现在主对话里），成员之间不能直接通信，产出都回到主理人。
//! 成员的对话只在这一轮回答里存在；下一条用户消息需要时由主理人重新 TeamCreate。
use super::*;
use agent_kernel::builtin::{Outbox, Team, TeamMember};
use agent_skill_discovery::plugin;

/// 成员人设最多放这么多字，和主理人人设同一个上限。
const MAX_MEMBER_CHARS: usize = 24_000;
/// 成员 agent md 没写 `maxTurns` 时，一个成员一次任务最多走这么多步。
const DEFAULT_MEMBER_TURNS: usize = 40;

pub struct Member {
    pub info: TeamMember,
    persona: String,
    max_turns: Option<u32>,
}

pub struct Setup {
    pub lead: TeamMember,
    pub members: Vec<Member>,
}

fn zh(value: &Option<plugin::Bilingual>) -> String {
    value.as_ref().map(|b| b.pick(true).to_string()).unwrap_or_default()
}

/// 读出对话绑定的专家团：主理人和每个成员。不是专家团、成员读不出来时返回 `None`；个别成员缺文件就跳过它。
pub fn load(plugins: &Path, binding: &sessions::ExpertBinding) -> Option<Setup> {
    let receipt = plugin::read_receipt(plugins, &binding.plugin).ok()?;
    if receipt["kind"].as_str() != Some("team") {
        return None;
    }
    let manifest: Value = fs::read(plugins.join(&binding.plugin).join(".codebuddy-plugin/plugin.json"))
        .or_else(|_| fs::read(plugins.join(&binding.plugin).join(".workbuddy-plugin/plugin.json")))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())?;
    let listed: Vec<&Value> = manifest["members"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
    let pick = |id: &str, field: &str| -> String {
        listed.iter().find(|m| m["id"].as_str() == Some(id)).and_then(|m| m[field]["zh"].as_str().or(m[field].as_str())).unwrap_or_default().to_string()
    };
    let mut ids: Vec<String> = manifest["teamInfo"]["memberAgents"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default();
    ids.retain(|id| *id != binding.agent);
    let mut members = Vec::new();
    for id in ids {
        let Ok(doc) = plugin::read_agent(plugins, &binding.plugin, &id) else { continue };
        let name = [zh(&doc.display_name), pick(&id, "name")].into_iter().find(|n| !n.is_empty()).unwrap_or_else(|| id.clone());
        let profession = [zh(&doc.profession), pick(&id, "profession")].into_iter().find(|n| !n.is_empty()).unwrap_or_default();
        let persona = member_persona(&name, &profession, &binding.name, &doc);
        members.push(Member { info: TeamMember::new(id, name, profession, doc.description.trim()), persona, max_turns: doc.max_turns });
    }
    if members.is_empty() {
        return None;
    }
    Some(Setup { lead: TeamMember::new(binding.agent.clone(), binding.name.clone(), binding.profession.clone(), ""), members })
}

/// 成员的系统提示词：它是谁、向谁汇报，以及人设正文。
fn member_persona(name: &str, profession: &str, lead: &str, doc: &plugin::AgentDoc) -> String {
    let who = if profession.is_empty() { name.to_string() } else { format!("{name}，{profession}") };
    let mut body: String = doc.body.trim().chars().take(MAX_MEMBER_CHARS).collect();
    if doc.body.trim().chars().count() > MAX_MEMBER_CHARS {
        body.push_str("\n…（人设过长，后面的内容已省略）");
    }
    let mut text = format!(
        "## 你的角色\n你是专家团的成员「{who}」，主理人是「{lead}」。主理人交给你一项独立任务，请按下面的人设和方法完成它。\n\n{body}\n\n\
         ## 协作方式\n- 你只能和主理人联系；不能联系其他成员，其他成员的产出由主理人转交给你。\n\
         - 主理人看不到你的工具调用和文件，所以把完整的产出（结论、正文、文件路径）写进回复：可以用 `SendMessage`（to 填 `lead`）发给主理人，也可以直接作为最终回答。\n\
         - 只做交给你的这一件事，不要替别的成员做他们的专业产出。\n\
         - 你不增加本轮没有的工具、权限或记忆；需要用户确认的操作照常会询问用户。"
    );
    if !doc.skills.is_empty() {
        text.push_str(&format!("\n\n你预装的技能：{}。需要时用 `load_skill` 加载后照着做。", doc.skills.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join("、")));
    }
    text
}

/// 追加在主理人人设后面的团队章程：WorkBuddy 专家团规范里的协作铁律，加上工具怎么用和成员名单。
pub fn charter(lead: &TeamMember, members: &[Member]) -> String {
    let mut text = String::from(
        "## 你的专家团\n你是这个专家团的主理人，成员可以作为独立的协作方为你工作。\n\n\
         ### 协作铁律\n1. **先建团队**：开始调度前由你自己调用 `TeamCreate`；团队创建只能由你执行，不能委派给成员。\n\
         2. **成员的产出来自成员**：按 SOP 阶段用 `Agent` 工具把成员拉进来，下发独立任务；专业产出必须由对应成员写出，你只做编排、转交和汇编，不要自己代写或模拟成员发言。\n\
         3. **信息经你中转**：成员的产出回到你这里，由你汇总后转交下一阶段；成员之间不能直接通信。每个阶段结束后，把前序成员的完整产出原文放进下一阶段成员的任务描述。\n\
         4. **以成员的结论为准**：采信前先让对应成员给出专业结论；裁决型角色必须给出明确结论。\n\n\
         ### 红线\n- 不要跳过 `TeamCreate`，自己并行写出多个角色的内容。\n- 不要代写任何成员的专业产出。\n- 前序阶段没有完成，不要跳到后续阶段。\n- 不要让成员互相直连。\n- 不要调度主理人自己（编排、汇总、决策由你亲自完成）。\n\n\
         ### 工具\n- `TeamCreate`：每次回答开始调度前先建团队（团队只在这一轮回答里存在，下一条用户消息需要时重新建）。\n\
         - `Agent`：`name` 和 `subagent_type` 都填成员的 agent id（见下表，不要用中文名）；`prompt` 要写全任务、所需输入和希望收到什么。**同一阶段没有数据依赖的成员，在同一条回复里一起调用，它们会同时工作。**\n\
         - `SendMessage`：给已经工作过的成员追加要求或回答它的问题，它会接着自己之前的上下文做。\n\
         - `TeamDelete`：全部交付后解散团队。\n\
         每个阶段结束后向用户简要通报进度。\n\n### 成员\n",
    );
    for member in members {
        let m = &member.info;
        text.push_str(&format!("- `{}`：{}{}", m.id, m.name, if m.profession.is_empty() { String::new() } else { format!("（{}）", m.profession) }));
        if !m.description.trim().is_empty() {
            text.push_str(&format!(" — {}", m.description.trim()));
        }
        text.push('\n');
    }
    text.push_str(&format!("\n你是「{}」（agent id `{}`）。", lead.name, lead.id));
    text
}

/// 成员 agent 要用到的应用资源：数据目录和与主对话共用的权限检查。
#[derive(Clone)]
pub struct Env {
    pub dirs: Dirs,
    pub permission: Arc<Permission>,
}

/// 主理人的四个团队工具。成员 agent 在第一次被调度时才建。
pub fn tools(
    env: &Env, setup: Setup, provider: Arc<dyn ModelProvider>, workspace: PathBuf, budget: agent_kernel::model::TokenBudget,
    events: Option<Arc<dyn Fn(&str, &agent_kernel::Event) + Send + Sync>>,
) -> Result<Vec<Arc<dyn agent_kernel::Tool>>, String> {
    let Setup { lead, members } = setup;
    let infos: Vec<TeamMember> = members.iter().map(|m| m.info.clone()).collect();
    let by_id: HashMap<String, (String, Option<u32>)> = members.into_iter().map(|m| (m.info.id.clone(), (m.persona, m.max_turns))).collect();
    let runs = env.dirs.runs();
    let logs: Mutex<HashMap<String, JsonlRunLog>> = Mutex::new(HashMap::new());
    let journal = move |record: &agent_kernel::RunEvent| -> Result<(), agent_kernel::HarnessError> {
        let parent = record.session_id.split('#').next().unwrap_or_default();
        if !sessions::valid_id(parent) {
            return Ok(());
        }
        lock(&logs).entry(parent.to_string()).or_insert_with(|| JsonlRunLog::new(runs.join(parent))).append(record)
    };
    let (build_env, release_env) = (env.clone(), env.clone());
    let factory = move |member: &TeamMember, outbox: &Outbox| -> Result<Agent, String> {
        let (persona, max_turns) = by_id.get(&member.id).cloned().ok_or("成员不在团队里")?;
        member_agent(&build_env, provider.clone(), &workspace, budget, &persona, max_turns, outbox)
    };
    let team = Team::new(lead, infos, factory)
        .observe_run(journal)
        .on_finish(move |session| release_env.permission.forget_session(session));
    let team = Arc::new(match events {
        Some(events) => team.observe(move |session, _member, event| events(session, event)),
        None => team,
    });
    Ok(team.tools())
}

/// 一个成员的 agent：人设做系统提示词，文件、命令、联网和技能工具与主对话一样，同一个权限检查。
fn member_agent(
    env: &Env, provider: Arc<dyn ModelProvider>, workspace: &Path, budget: agent_kernel::model::TokenBudget, persona: &str, max_turns: Option<u32>,
    outbox: &Outbox,
) -> Result<Agent, String> {
    let dirs = &env.dirs;
    let skills = Arc::new(skills_for(dirs, dirs.chat.join("bundled-skills")));
    fs::create_dir_all(dirs.spill()).map_err(|e| format!("无法创建 {}：{e}", dirs.spill().display()))?;
    let roots = vec![workspace.to_path_buf(), dirs.spill()];
    let system = format!("{persona}\n\n## 工作环境\n操作系统：{}；工作目录：{}。", std::env::consts::OS, workspace.display());
    let mut builder = Agent::builder(provider, Arc::new(agent_kernel::MemoryStore::new()))
        .system_prompt(system)
        .prompt_section(skills.prompt_section())
        .tool(Shell::new(workspace).sandbox(sandbox_gate::hook(env.permission.clone(), workspace)))
        .tool(WebFetch::new())
        .tool_arc(outbox.tool())
        // 与主对话同一套检查：先是用户自己的钩子，再是权限（确认卡片出现在主对话里），然后是检查点、防重复和长输出落盘。
        .middleware(hooks::Hooks::new(dirs.chat.join("hooks.json"), workspace))
        .middleware(env.permission.clone())
        .middleware(checkpoints::Checkpoints::new(dirs.checkpoints(), workspace))
        .middleware(RepeatGuard::new().ignore_tool("process_read"))
        .middleware(SpillOutput::new(dirs.spill()))
        .middleware(ContextTrim::new(budget.context_limit).reserve_output(budget.trim_headroom))
        .retry(agent_kernel::RetryPolicy { max_retries: 3, ..Default::default() })
        .max_turns(max_turns.map_or(DEFAULT_MEMBER_TURNS, |t| t as usize));
    let mut readable = fs_tools(roots.clone()).map_err(|e| e.to_string())?;
    readable.retain(|tool| tool.spec().name != "read_file");
    readable.push(Arc::new(ReadFile::new(roots, AttachmentStore::new(dirs.chat.join("attachments"))).map_err(|e| e.to_string())?));
    let writable = fs_write_tools(vec![workspace.to_path_buf()]).map_err(|e| e.to_string())?;
    for tool in readable.into_iter().chain(writable).chain(skills.tools()) {
        builder = builder.tool_arc(tool);
    }
    builder.build().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    fn install(root: &Path, with_members: bool) -> sessions::ExpertBinding {
        let manifest = json!({
            "name": "dev-team", "version": "1.0.0", "description": "d", "expertType": "team", "agentName": "dev-team-lead",
            "agents": ["./agents/dev-team-lead.md", "./agents/xu-pm.md", "./agents/kou-dev.md"],
            "teamInfo": { "leadAgent": "dev-team-lead", "memberAgents": if with_members { json!(["xu-pm", "kou-dev", "ghost"]) } else { json!([]) } },
            "members": [
                { "id": "dev-team-lead", "name": {"en": "Qi", "zh": "齐活林"}, "profession": {"en": "Director", "zh": "交付总监"}, "role": "lead" },
                { "id": "xu-pm", "name": {"en": "Xu", "zh": "许清楚"}, "profession": {"en": "PM", "zh": "产品经理"}, "role": "member" }
            ]
        });
        let agent = |name: &str, body: &str| format!("---\nname: {name}\ndescription: {name}负责的事\nmaxTurns: 12\n---\n{body}");
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (path, data) in [
            (".codebuddy-plugin/plugin.json", manifest.to_string()),
            ("agents/dev-team-lead.md", agent("dev-team-lead", "# 交付总监\n按 SOP 调度。")),
            ("agents/xu-pm.md", agent("xu-pm", "# 产品经理\n先澄清需求。")),
            ("agents/kou-dev.md", agent("kou-dev", "# 工程师\n写代码。")),
        ] {
            w.start_file(path, zip::write::SimpleFileOptions::default()).unwrap();
            w.write_all(data.as_bytes()).unwrap();
        }
        let pkg = plugin::inspect_zip(&w.finish().unwrap().into_inner()).unwrap();
        plugin::install(&root.join("plugins"), &root.join("backups"), &pkg, "local", json!({})).unwrap();
        sessions::ExpertBinding { plugin: "dev-team".into(), agent: "dev-team-lead".into(), name: "齐活林".into(), profession: "交付总监".into() }
    }

    #[test]
    fn a_team_loads_its_members_with_names_from_their_agent_files_or_the_manifest_and_skips_missing_ones() {
        let root = std::env::temp_dir().join(format!("team-load-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let binding = install(&root, true);
        let setup = load(&root.join("plugins"), &binding).unwrap();
        assert_eq!(setup.lead.id, "dev-team-lead");
        let ids: Vec<&str> = setup.members.iter().map(|m| m.info.id.as_str()).collect();
        assert_eq!(ids, vec!["xu-pm", "kou-dev"], "ghost has no agent file and the lead is never a member");
        assert_eq!((setup.members[0].info.name.as_str(), setup.members[0].info.profession.as_str()), ("许清楚", "产品经理"), "named by the manifest's members list");
        assert_eq!(setup.members[1].info.name, "kou-dev", "no name anywhere: the id");
        assert_eq!(setup.members[0].info.description, "xu-pm负责的事");
        assert_eq!(setup.members[0].max_turns, Some(12));
        assert!(setup.members[0].persona.contains("「许清楚，产品经理」") && setup.members[0].persona.contains("主理人是「齐活林」") && setup.members[0].persona.contains("先澄清需求"));
        assert!(setup.members[0].persona.contains("不能联系其他成员") && setup.members[0].persona.contains("不增加本轮没有的工具"));
        // A single expert, a team without members and a removed plugin are not teams.
        assert!(load(&root.join("plugins"), &sessions::ExpertBinding { plugin: "nope".into(), ..binding.clone() }).is_none());
        plugin::remove(&root.join("plugins"), "dev-team").unwrap();
        let empty = install(&root, false);
        assert!(load(&root.join("plugins"), &empty).is_none());
        let _ = fs::remove_dir_all(&root);
    }

    /// 整条链路：主理人建团队、调度成员，成员是用同一套工具和权限建出来的真 agent，把产出发回主理人。
    #[test]
    fn a_lead_calls_a_member_built_from_the_real_tools_and_the_report_comes_back() {
        use agent_kernel::provider::scripted::{ScriptedProvider, Step};
        let root = std::env::temp_dir().join(format!("team-run-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (work, data) = (root.join("work"), root.join("data"));
        fs::create_dir_all(&work).unwrap();
        fs::create_dir_all(&data).unwrap();
        let env = Env { dirs: Dirs { chat: data.join("chat") }, permission: Arc::new(Permission::new(PermissionMode::Auto, |_: &ApprovalRequest<'_>| Approval::Once).children_inherit()) };
        let dirs = env.dirs.clone();
        let binding = install(&root.join("unused"), true);
        // 同一份插件装进应用的插件目录。
        fs::create_dir_all(dirs.plugins()).unwrap();
        copy_dir(&root.join("unused/plugins/dev-team"), &dirs.plugins().join("dev-team"));
        let setup = load(&dirs.plugins(), &binding).unwrap();
        let provider = Arc::new(ScriptedProvider::new(vec![
            Step::tool_call("c1", "TeamCreate", r#"{"team_name":"交付团队"}"#),
            Step::tool_call("c2", "Agent", r#"{"name":"xu-pm","subagent_type":"xu-pm","prompt":"请澄清需求：做登录页"}"#),
            // 成员自己的两步：先把产出发给主理人，再收尾。
            Step::tool_call("m1", "SendMessage", r#"{"to":"lead","message":"需求已澄清：邮箱登录、找回密码、记住我。"}"#),
            Step::text("已发给主理人"),
            Step::text("团队已完成"),
        ]));
        let tools = tools(&env, setup, provider.clone(), work.clone(), agent_kernel::model::TokenBudget::resolve(200_000, None), None).unwrap();
        assert_eq!(tools.iter().map(|t| t.spec().name.clone()).collect::<Vec<_>>(), ["TeamCreate", "Agent", "SendMessage", "TeamDelete"]);
        let mut builder = Agent::builder(provider.clone(), Arc::new(agent_kernel::MemoryStore::new())).max_turns(10);
        for tool in tools {
            builder = builder.tool_arc(tool);
        }
        let lead = builder.build().unwrap();
        let summary = lead.run("s1", "做一个登录页", &mut |_| {}, &Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(summary.text, "团队已完成");
        let requests = provider.requests();
        // 成员那次请求：系统提示词是它的人设，工具是真的文件/命令/联网工具加上发给主理人的 SendMessage，不含团队工具。
        let member = requests.iter().find(|r| r.messages.iter().any(|m| m.text().contains("请澄清需求"))&& r.messages[0].role == agent_kernel::Role::System && r.messages[0].text().contains("许清楚")).expect("a request by the member");
        let names: Vec<&str> = member.tools.iter().map(|t| t.name.as_str()).collect();
        for expected in ["SendMessage", "shell", "write_file", "read_file", "web_fetch"] {
            assert!(names.contains(&expected), "{expected} in {names:?}");
        }
        assert!(!names.contains(&"TeamCreate") && !names.contains(&"Agent"), "{names:?}");
        assert!(member.messages[0].text().contains("先澄清需求") && member.messages[0].text().contains(&work.display().to_string()));
        // 回到主理人的是成员发来的产出。
        let last = requests.last().unwrap();
        let report = last.messages.iter().rev().find(|m| m.role == agent_kernel::Role::Tool).unwrap().text();
        assert!(report.contains("[Report from 许清楚（产品经理）]") && report.contains("邮箱登录、找回密码、记住我"), "{report}");
        let _ = fs::remove_dir_all(&root);
    }

    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap().flatten() {
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() { copy_dir(&entry.path(), &target) } else { fs::copy(entry.path(), target).unwrap(); }
        }
    }

    #[test]
    fn the_charter_carries_the_collaboration_rules_and_lists_members_by_agent_id() {
        let members = vec![Member { info: TeamMember::new("xu-pm", "许清楚", "产品经理", "把需求说清楚"), persona: String::new(), max_turns: None }];
        let text = charter(&TeamMember::new("dev-team-lead", "齐活林", "交付总监", ""), &members);
        for must in ["TeamCreate", "同一条回复里一起调用", "成员之间不能直接通信", "不要代写任何成员的专业产出", "- `xu-pm`：许清楚（产品经理） — 把需求说清楚", "agent id `dev-team-lead`"] {
            assert!(text.contains(must), "{must}\n{text}");
        }
    }
}
