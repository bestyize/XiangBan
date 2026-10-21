//! Skill acquisition policy: search only for a user request or a justified capability gap.
//! This middleware does not install anything and cannot grant script execution permission.
use agent_kernel::{
    FinishReason, Message, Middleware, Nudge, Role, Stop, StopDecision, ToolCall, ToolCtx,
    ToolGate, ToolResult, ToolSpec, Turn,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

const MAX_SEARCHES_PER_TASK: usize = 3;
const MAX_TRACKED_SESSIONS: usize = 128;

#[derive(Default)]
struct TaskState {
    user_index: usize,
    user_text: String,
    searched: HashSet<String>,
    failures: HashMap<String, HashSet<String>>,
    nudged: bool,
}

#[derive(Default)]
pub struct AcquisitionPolicy {
    tasks: Mutex<HashMap<String, TaskState>>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn user_requests_skill(text: &str) -> bool {
    let lower = text.to_lowercase();
    let mentions_skill = lower.contains("skill") || lower.contains("技能");
    let asks_to_find = [
        "找", "搜索", "查找", "寻找", "推荐", "安装", "获取", "下载",
        "find", "search", "install", "discover", "recommend",
    ].iter().any(|word| lower.contains(word));
    mentions_skill && asks_to_find
}

fn forbids_online_search(text: &str) -> bool {
    let lower = text.to_lowercase();
    ["不要联网", "禁止联网", "不允许联网", "不要在线搜索", "离线完成",
        "no internet", "don't browse", "do not browse", "offline only",
        "no web search"].iter().any(|word| lower.contains(word))
}

fn environmental_failure(error: &str) -> bool {
    let lower = error.to_lowercase();
    [
        "permission", "denied", "not authorized", "unauthorized", "forbidden",
        "accessibility", "tcc", "rate limit", "quota", "429", "401", "403",
        "timeout", "timed out", "connection", "network", "dns", "offline",
        "invalid argument", "invalid parameter", "missing api key",
        "权限", "拒绝", "授权", "配额", "限流", "网络", "超时", "认证", "参数错误",
    ].iter().any(|marker| lower.contains(marker))
}

fn recent_nonenvironment_failures(state: &TaskState) -> bool {
    state.failures.values().any(|calls| calls.len() >= 2)
}

fn assistant_reports_inability(text: &str) -> bool {
    let lower = text.to_lowercase();
    [
        "做不到", "无法完成", "无法处理", "能力不足", "缺少工具", "不支持",
        "未能完成", "暂时无法", "无法执行", "不能完成",
        "cannot", "unable to", "unsupported", "no available tool", "can't complete",
    ].iter().any(|phrase| lower.contains(phrase))
}

impl Middleware for AcquisitionPolicy {
    fn before_model(&self, turn: &mut Turn) -> Result<(), agent_kernel::HarnessError> {
        let Some((index, user)) = turn.messages.iter().enumerate().rev()
            .find(|(_, m)| m.role == Role::User && !m.synthetic) else {
            return Ok(());
        };
        let text = user.text();
        let mut tasks = lock(&self.tasks);
        if tasks.len() >= MAX_TRACKED_SESSIONS && !tasks.contains_key(&turn.session) {
            // Only transient in-memory hints; eviction cannot grant permissions.
            if let Some(first) = tasks.keys().next().cloned() {
                tasks.remove(&first);
            }
        }
        let task = tasks.entry(turn.session.clone()).or_default();
        if task.user_index != index || task.user_text != text {
            *task = TaskState {
                user_index: index,
                user_text: text,
                ..TaskState::default()
            };
        }
        Ok(())
    }

    fn after_tool(&self, session: &str, call: &ToolCall, result: &mut ToolResult) {
        if !result.is_error || environmental_failure(&result.output)
            || call.name.starts_with("skill_")
            || matches!(call.name.as_str(), "todo" | "ask_user" | "time_now") {
            return;
        }
        if let Some(task) = lock(&self.tasks).get_mut(session) {
            // Distinct tool_call IDs, not error log lines or retries of the same result.
            task.failures.entry(call.name.clone()).or_default().insert(call.id.clone());
        }
    }

    fn before_tool(&self, ctx: &ToolCtx, _call: &ToolCall, args: &Value, spec: &ToolSpec) -> ToolGate {
        if spec.name != "skill_search" {
            return ToolGate::Proceed;
        }
        let mut tasks = lock(&self.tasks);
        let Some(state) = tasks.get_mut(ctx.session()) else {
            return ToolGate::Block("Skill search needs an active user task.".into());
        };
        if forbids_online_search(&state.user_text) {
            return ToolGate::Block("The user requested offline work; do not search an online Skill registry.".into());
        }
        let keyword = args.get("keyword").and_then(Value::as_str).unwrap_or("").trim().to_lowercase();
        if keyword.len() < 2 || keyword.len() > 80 {
            return ToolGate::Block("Skill search keyword must be 2–80 characters.".into());
        }
        let explicit = user_requests_skill(&state.user_text);
        if !explicit {
            let reason = args.get("reason").and_then(Value::as_str).unwrap_or("");
            let gap = args.get("gap").and_then(Value::as_str).unwrap_or("").trim();
            let alternatives = args.get("alternatives").and_then(Value::as_str).unwrap_or("").trim();
            if gap.chars().count() < 8 || alternatives.chars().count() < 8 {
                return ToolGate::Block(
                    "Before proactive Skill search, identify the missing capability and which existing tools/skills were considered (gap, alternatives). Do not search just because a task is difficult.".into()
                );
            }
            match reason {
                "capability_gap" => {}
                "repeated_failure" if recent_nonenvironment_failures(state) => {}
                "repeated_failure" => {
                    return ToolGate::Block(
                        "Repeated-failure Skill search needs at least two actual, distinct non-environmental failures of the same tool during this task. First fix arguments, credentials, permissions, network or rate limits.".into()
                    )
                }
                _ => {
                    return ToolGate::Block(
                        "Proactive Skill search requires reason=capability_gap or reason=repeated_failure; explicit user requests need no reason.".into()
                    )
                }
            }
        }
        if state.searched.contains(&keyword) {
            return ToolGate::Block("Already searched this keyword in the current task; inspect results or try an existing tool instead.".into());
        }
        if state.searched.len() >= MAX_SEARCHES_PER_TASK {
            return ToolGate::Block("Skill search budget reached (three distinct searches per task). Compare existing candidates or ask the user.".into());
        }
        state.searched.insert(keyword);
        ToolGate::Proceed
    }

    fn before_stop(&self, stop: &Stop<'_>) -> StopDecision {
        if stop.finish != FinishReason::Stop {
            return StopDecision::Finish;
        }
        let mut tasks = lock(&self.tasks);
        let Some(state) = tasks.get_mut(stop.session) else { return StopDecision::Finish };
        if state.nudged || !state.searched.is_empty()
            || forbids_online_search(&state.user_text) || !recent_nonenvironment_failures(state) {
            return StopDecision::Finish;
        }
        let Some(message) = stop.history.last() else { return StopDecision::Finish };
        if message.role != Role::Assistant || !assistant_reports_inability(&message.text()) {
            return StopDecision::Finish;
        }
        state.nudged = true;
        StopDecision::Continue(Nudge {
            reason: "skill_capability_gap".into(),
            message: "本轮已有同一工具的多次真实失败，且你表示无法完成任务。仅在确认不是权限、参数、网络、配额或环境错误、现有工具与已安装 Skill 确实不能解决时，才调用 skill_search(reason=repeated_failure, gap=具体能力缺口, alternatives=已检查的现有方法, keyword=相关关键词)，并先审查候选技能。否则解释真正的失败原因。第三方 Skill 安装和脚本执行均需用户单独批准。".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::{Message, Risk, ToolSpec};

    fn policy_with_task(text: &str) -> AcquisitionPolicy {
        let policy = AcquisitionPolicy::default();
        let mut turn = Turn {
            session: "s".into(), index: 0,
            messages: vec![Message::user_text(text)],
            tools: vec![],
        };
        policy.before_model(&mut turn).unwrap();
        policy
    }

    fn search(policy: &AcquisitionPolicy, args: Value) -> ToolGate {
        let spec = ToolSpec::new("skill_search", "search", serde_json::json!({})).risk(Risk::Network);
        let call = ToolCall { id: "search".into(), name: "skill_search".into(), arguments: "{}".into() };
        policy.before_tool(&ToolCtx::detached("s"), &call, &args, &spec)
    }

    fn is_allowed(gate: ToolGate) -> bool {
        matches!(gate, ToolGate::Proceed)
    }

    #[test]
    fn explicit_user_request_still_works_without_extra_arguments() {
        let policy = policy_with_task("帮我找一个处理 Excel 的 Skill");
        assert!(is_allowed(search(&policy, serde_json::json!({"keyword":"Excel"}))));
        assert!(!is_allowed(search(&policy, serde_json::json!({"keyword":"excel"}))));
    }

    #[test]
    fn autonomous_discovery_requires_specific_gap_and_alternatives() {
        let policy = policy_with_task("帮我整理文件");
        assert!(!is_allowed(search(&policy, serde_json::json!({"keyword":"excel"}))));
        assert!(!is_allowed(search(&policy, serde_json::json!({
            "keyword":"excel", "reason":"capability_gap", "gap":"没有可用的电子表格分析工具",
            "alternatives":"试过的方案只有内置的文件读取，不足以操作公式"
        }))) == false);
        // The first blocked search must not consume the search budget.
        assert!(!is_allowed(search(&policy, serde_json::json!({"keyword":"excel"}))));
    }

    #[test]
    fn repeated_failure_requires_observed_calls_and_excludes_environment_errors() {
        let policy = policy_with_task("批量转换这种特殊文件格式");
        let args = serde_json::json!({
            "keyword":"file converter", "reason":"repeated_failure",
            "gap":"当前没有处理这种格式的转换工具",
            "alternatives":"尝试通过原来的转换工具处理格式"
        });
        assert!(!is_allowed(search(&policy, args.clone())));
        for id in ["1", "2"] {
            let call = ToolCall { id:id.into(), name:"read_file".into(), arguments:"{}".into() };
            let mut result = ToolResult::error("unsupported proprietary format");
            policy.after_tool("s", &call, &mut result);
        }
        assert!(is_allowed(search(&policy, args)));
    }

    #[test]
    fn permission_errors_never_qualify_as_capability_failures() {
        let policy = policy_with_task("转换文件");
        for id in ["1", "2"] {
            let call = ToolCall { id:id.into(), name:"read_file".into(), arguments:"{}".into() };
            let mut result = ToolResult::error("permission denied");
            policy.after_tool("s", &call, &mut result);
        }
        assert!(!is_allowed(search(&policy, serde_json::json!({
            "keyword":"file converter", "reason":"repeated_failure",
            "gap":"当前无法读取受系统保护的文件", "alternatives":"现有文件读取工具提示权限不足"
        }))));
    }

    #[test]
    fn offline_instruction_blocks_even_explicit_search() {
        let policy = policy_with_task("不要联网，帮我找一个 Skill");
        assert!(!is_allowed(search(&policy, serde_json::json!({"keyword":"excel"}))));
    }

    #[test]
    fn a_new_user_turn_resets_the_keyword_budget() {
        let policy = policy_with_task("帮我找一个 Skill");
        assert!(is_allowed(search(&policy, serde_json::json!({"keyword":"excel"}))));
        let mut turn = Turn {
            session:"s".into(),index:1,
            messages:vec![Message::user_text("帮我找一个 Skill"), Message::user_text("再帮我找一个 Skill")],
            tools:vec![],
        };
        policy.before_model(&mut turn).unwrap();
        assert!(is_allowed(search(&policy, serde_json::json!({"keyword":"excel"}))));
    }

    #[test]
    fn nudge_only_after_real_failures_and_inability() {
        let policy = policy_with_task("转换特殊文件");
        for id in ["1","2"] {
            let call = ToolCall { id:id.into(),name:"convert_file".into(),arguments:"{}".into() };
            let mut result = ToolResult::error("unsupported format");
            policy.after_tool("s", &call, &mut result);
        }
        let history = vec![Message::user_text("转换特殊文件"), Message::assistant("当前无法完成转换", vec![])];
        let stop = Stop {
            session:"s",history:&history,run_start:0,finish:FinishReason::Stop,
            continuations:0,continuations_without_progress:0,
        };
        assert!(matches!(policy.before_stop(&stop), StopDecision::Continue(_)));
        assert!(matches!(policy.before_stop(&stop), StopDecision::Finish));
    }
}
