mod common;

use agent_kernel::builtin::{Team, TeamMember};
use agent_kernel::provider::scripted::{ScriptedProvider, Step};
use agent_kernel::*;
use common::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Staff {
    providers: Mutex<HashMap<String, Arc<ScriptedProvider>>>,
    stores: Mutex<HashMap<String, Arc<MemoryStore>>>,
    finished: Mutex<Vec<String>>,
}

fn member(id: &str, name: &str, profession: &str) -> TeamMember {
    TeamMember::new(id, name, profession, format!("{profession}负责{id}的工作"))
}

/// A team whose members play the given scripts. Members get their own `SendMessage` tool plus any extra tools.
fn team_of(scripts: Vec<(&str, Vec<Step>)>, extra: impl Fn(AgentBuilder) -> AgentBuilder + Send + Sync + 'static) -> (Arc<Team>, Arc<Staff>) {
    let staff = Arc::new(Staff { providers: Mutex::default(), stores: Mutex::default(), finished: Mutex::default() });
    let mut scripts: HashMap<String, Vec<Step>> = scripts.into_iter().map(|(id, steps)| (id.to_string(), steps)).collect();
    let members = vec![member("xu-pm", "许清楚", "产品经理"), member("kou-dev", "寇豆码", "工程师"), member("yan-qa", "严过关", "QA工程师")];
    let supply = Mutex::new(std::mem::take(&mut scripts));
    let record = staff.clone();
    let done = staff.clone();
    let team = Team::new(member("qi-lead", "齐活林", "交付总监"), members, move |who, outbox| {
        let steps = supply.lock().unwrap().remove(&who.id).ok_or_else(|| format!("no script for {}", who.id))?;
        let provider = Arc::new(ScriptedProvider::new(steps));
        let store = Arc::new(MemoryStore::new());
        record.providers.lock().unwrap().insert(who.id.clone(), provider.clone());
        record.stores.lock().unwrap().insert(who.id.clone(), store.clone());
        let builder = Agent::builder(provider, store).system_prompt(format!("You are {}.", who.name)).tool_arc(outbox.tool()).retry(fast_retry());
        extra(builder).build().map_err(|e| e.to_string())
    })
    .on_finish(move |session| done.finished.lock().unwrap().push(session.to_string()));
    (Arc::new(team), staff)
}

fn lead(team: &Arc<Team>, steps: Vec<Step>) -> Harness {
    harness(steps, |mut b| {
        for tool in team.tools() {
            b = b.tool_arc(tool);
        }
        b
    })
}

fn results(h: &Harness) -> Vec<String> {
    tool_messages(h).into_iter().map(|(_, text)| text).collect()
}

/// Tool results after `TeamCreate`'s.
fn reports(h: &Harness) -> Vec<String> {
    results(h).into_iter().skip(1).collect()
}

fn create() -> Step {
    Step::tool_call("tc", "TeamCreate", r#"{"team_name":"交付团队"}"#)
}

#[test]
fn the_lead_creates_the_team_and_members_report_back_with_their_own_words() {
    let (team, staff) = team_of(vec![("xu-pm", vec![Step::text("需求已澄清：共三条。")])], |b| b);
    let h = lead(&team, vec![
        create(),
        Step::tool_call("a1", "Agent", r#"{"name":"xu-pm","subagent_type":"xu-pm","prompt":"请澄清需求","description":"需求"}"#),
        Step::text("汇总完毕"),
    ]);
    let (result, _) = run(&h.agent, "做一个登录页");
    assert_eq!(result.unwrap().text, "汇总完毕");
    let requests = h.provider.requests();
    let roster = requests[1].messages.iter().rev().find(|m| m.role == Role::Tool).unwrap().text();
    assert!(roster.contains("交付团队") && roster.contains("`xu-pm`") && roster.contains("许清楚（产品经理）") && roster.contains("`kou-dev`"), "{roster}");
    assert!(!roster.contains("`qi-lead`: "), "the lead is not listed as a member: {roster}");
    let report = reports(&h).remove(0);
    assert_eq!(report, "[Report from 许清楚（产品经理）]\n需求已澄清：共三条。");
    // The member saw only its own prompt, in its own session under the lead's.
    let seen = &staff.providers.lock().unwrap()["xu-pm"].requests()[0].messages;
    assert_eq!((seen[0].text().as_str(), seen[1].text().as_str()), ("You are 许清楚.", "请澄清需求"));
    let ids = staff.stores.lock().unwrap()["xu-pm"].session_ids();
    assert!(ids.len() == 1 && ids[0].starts_with("s1#team-xu-pm-"), "{ids:?}");
    // The lead's own history does not carry the member's working.
    assert!(h.store.load("s1").unwrap().iter().all(|m| !m.text().contains("You are 许清楚")));
}

#[test]
fn members_started_in_one_reply_work_at_the_same_time() {
    let nap = |b: AgentBuilder| b.tool(FnTool::new(spec("nap"), |_, _| { std::thread::sleep(Duration::from_millis(300)); Ok("rested".into()) }));
    let (team, _) = team_of(
        vec![
            ("xu-pm", vec![Step::tool_call("n", "nap", "{}"), Step::text("产品完成")]),
            ("kou-dev", vec![Step::tool_call("n", "nap", "{}"), Step::text("开发完成")]),
        ],
        nap,
    );
    let h = lead(&team, vec![
        create(),
        Step::tool_call("a1", "Agent", r#"{"name":"xu-pm","prompt":"p"}"#).and_tool_call("a2", "Agent", r#"{"name":"kou-dev","prompt":"d"}"#),
        Step::text("ok"),
    ]);
    let started = Instant::now();
    assert!(run(&h.agent, "go").0.is_ok());
    assert!(started.elapsed() < Duration::from_millis(560), "both members napped together: {:?}", started.elapsed());
    let out = reports(&h);
    assert!(out[0].contains("产品完成") && out[1].contains("开发完成"), "{out:?}");
}

#[test]
fn nothing_starts_before_the_team_exists_and_only_listed_members_can_be_started() {
    let (team, staff) = team_of(vec![("xu-pm", vec![Step::text("x")])], |b| b);
    let h = lead(&team, vec![
        Step::tool_call("a0", "Agent", r#"{"name":"xu-pm","prompt":"p"}"#),
        create(),
        Step::tool_call("a1", "Agent", r#"{"name":"qi-lead","prompt":"p"}"#),
        Step::tool_call("a2", "Agent", r#"{"name":"无此人","prompt":"p"}"#),
        Step::tool_call("a3", "Agent", r#"{"name":"xu-pm","subagent_type":"kou-dev","prompt":"p"}"#),
        Step::tool_call("a4", "TeamCreate", r#"{"team_name":"again"}"#),
        Step::text("done"),
    ]);
    assert!(run(&h.agent, "go").0.is_ok());
    // Each call was its own round trip, so look through the whole history.
    let history: Vec<String> = h.store.load("s1").unwrap().iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect();
    assert!(history[0].contains("Call `TeamCreate` first"), "{history:?}");
    assert!(history[2].contains("team lead, which is you"), "{history:?}");
    assert!(history[3].contains("not a member") && history[3].contains("xu-pm, kou-dev, yan-qa"), "{history:?}");
    assert!(history[4].contains("must be the same agent id"), "{history:?}");
    assert!(history[5].contains("already exists"), "{history:?}");
    assert!(staff.providers.lock().unwrap().is_empty(), "no member was ever built");
}

#[test]
fn a_member_can_only_message_the_lead_and_what_it_sent_is_the_report() {
    let (team, _) = team_of(
        vec![(
            "kou-dev",
            vec![
                Step::tool_call("m1", "SendMessage", r#"{"to":"yan-qa","message":"帮我看下"}"#),
                Step::tool_call("m2", "SendMessage", r#"{"to":"lead","message":"代码已写完：login.js"}"#),
                Step::text("已发送"),
            ],
        )],
        |b| b,
    );
    let h = lead(&team, vec![create(), Step::tool_call("a1", "Agent", r#"{"name":"kou-dev","prompt":"写登录"}"#), Step::text("ok")]);
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(reports(&h).remove(0), "[Report from 寇豆码（工程师）]\n代码已写完：login.js");
}

#[test]
fn the_lead_continues_a_members_conversation_and_cannot_message_one_that_never_ran() {
    let (team, staff) = team_of(vec![("xu-pm", vec![Step::text("初稿"), Step::text("修订稿")])], |b| b);
    let h = lead(&team, vec![
        create(),
        Step::tool_call("s0", "SendMessage", r#"{"to":"xu-pm","message":"先别急"}"#),
        Step::tool_call("a1", "Agent", r#"{"name":"xu-pm","prompt":"写需求"}"#),
        Step::tool_call("s1", "SendMessage", r#"{"to":"xu-pm","message":"请补充验收标准"}"#),
        Step::text("done"),
    ]);
    assert!(run(&h.agent, "go").0.is_ok());
    let tools: Vec<String> = h.store.load("s1").unwrap().iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect();
    assert!(tools[1].contains("has not been started yet"), "{tools:?}");
    assert!(tools[2].ends_with("初稿"));
    assert!(tools[3].ends_with("修订稿"));
    // The follow-up reached the same conversation, which still holds the first job.
    let second = &staff.providers.lock().unwrap()["xu-pm"].requests()[1].messages;
    let texts: Vec<String> = second.iter().map(|m| m.text()).collect();
    assert!(texts.contains(&"写需求".to_string()) && texts.contains(&"初稿".to_string()) && texts.contains(&"请补充验收标准".to_string()), "{texts:?}");
    assert_eq!(staff.stores.lock().unwrap()["xu-pm"].session_ids().len(), 1);
}

#[test]
fn the_team_ends_with_the_run_or_on_delete_and_its_conversations_are_released() {
    let (team, staff) = team_of(vec![("xu-pm", vec![Step::text("a")]), ("kou-dev", vec![Step::text("b")])], |b| b);
    let h = lead(&team, vec![
        create(),
        Step::tool_call("a1", "Agent", r#"{"name":"xu-pm","prompt":"p"}"#),
        Step::tool_call("d", "TeamDelete", "{}"),
        Step::tool_call("a2", "Agent", r#"{"name":"xu-pm","prompt":"again"}"#),
        create(),
        Step::tool_call("a3", "Agent", r#"{"name":"kou-dev","prompt":"p"}"#),
        Step::text("done"),
    ]);
    assert!(run(&h.agent, "go").0.is_ok());
    let tools: Vec<String> = h.store.load("s1").unwrap().iter().filter(|m| m.role == Role::Tool).map(|m| m.text()).collect();
    assert!(tools[2].contains("Team ended (1 member conversation(s) closed)"), "{tools:?}");
    assert!(tools[3].contains("No team exists yet"), "{tools:?}");
    let finished = staff.finished.lock().unwrap().clone();
    assert_eq!(finished.len(), 2, "one release at TeamDelete, one when the run ended: {finished:?}");
    assert!(finished[0].starts_with("s1#team-xu-pm-") && finished[1].starts_with("s1#team-kou-dev-"));
    // A new run starts without a team.
    let again = lead(&team, vec![Step::tool_call("a", "Agent", r#"{"name":"xu-pm","prompt":"p"}"#), Step::text("x")]);
    assert!(run(&again.agent, "go").0.is_ok());
    assert!(results(&again).remove(0).contains("No team exists yet"));
}

#[test]
fn a_member_that_fails_becomes_an_error_result_for_the_lead() {
    let (team, _) = team_of(vec![("xu-pm", vec![Step::error(HarnessError::provider("model is down", false))])], |b| b);
    let h = lead(&team, vec![create(), Step::tool_call("a1", "Agent", r#"{"name":"xu-pm","prompt":"p"}"#), Step::text("ok")]);
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(reports(&h).remove(0), "许清楚（产品经理） failed: provider error: model is down");
}

#[test]
fn the_observer_sees_member_events_under_the_lead_session() {
    let seen: Arc<Mutex<Vec<(String, String)>>> = Arc::default();
    let log = seen.clone();
    let staff = Team::new(member("qi-lead", "齐活林", "交付总监"), vec![member("xu-pm", "许清楚", "产品经理")], |_, _| {
        Agent::builder(Arc::new(ScriptedProvider::new(vec![Step::chunks(&["he", "llo"])])), Arc::new(MemoryStore::new())).build().map_err(|e| e.to_string())
    })
    .observe(move |session, who, event| {
        if matches!(event, Event::TextDelta { .. }) {
            log.lock().unwrap().push((session.to_string(), who.to_string()));
        }
    });
    let team = Arc::new(staff);
    let h = lead(&team, vec![create(), Step::tool_call("a1", "Agent", r#"{"name":"xu-pm","prompt":"p"}"#), Step::text("ok")]);
    assert!(run(&h.agent, "go").0.is_ok());
    assert_eq!(*seen.lock().unwrap(), vec![("s1".to_string(), "xu-pm".to_string()); 2]);
}
