//! Tools allowing Xiaoban assistant to delegate complex tasks to external coding agents (Claude Code, Codex).

use agent_bridge::delegation::DelegationManager;
use agent_kernel::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct SimpleTool {
    spec: ToolSpec,
    run: Box<dyn Fn(&Value, &ToolCtx) -> Result<String, String> + Send + Sync>,
}

impl Tool for SimpleTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        (self.run)(&args, ctx)
    }
}

fn text_arg(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("缺少必要参数 {key}"))
}

fn optional_text(args: &Value, key: &str) -> Result<Option<String>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => {
            let t = s.trim();
            if t.is_empty() { Ok(None) } else { Ok(Some(t.to_string())) }
        }
        _ => Err(format!("参数 {key} 必须是字符串")),
    }
}

/// Create the suite of external agent delegation tools.
pub fn tools(
    manager: Arc<DelegationManager>,
    workspace: &Path,
) -> Vec<Arc<dyn Tool>> {
    let ws = workspace.to_path_buf();

    let delegate_mgr = manager.clone();
    let default_ws = ws.clone();
    let delegate_tool = SimpleTool {
        spec: ToolSpec::new(
            "agent_delegate",
            "把编程任务委派给本机已安装的外部 Coding Agent（Claude Code 或 Codex）在后台独立执行。当用户要求用 Claude Code / Codex、排查复杂 Bug、修测试、看大型项目或跑长任务时使用本工具。agent: claude 或 codex，也可填 auto 自动选择；prompt: 具体的任务提示词；workspace: 可选的工作目录路径，默认使用当前对话工作区。",
            json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "enum": ["claude", "codex", "auto"],
                        "description": "要委派的目标 Agent，可选 claude、codex 或 auto 自动选择"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "委派的具体任务描述或提示词"
                    },
                    "workspace": {
                        "type": "string",
                        "description": "可选的工作目录路径，默认使用当前对话工作区"
                    }
                },
                "required": ["agent", "prompt"]
            }),
        )
        .risk(Risk::Exec)
        .self_managed()
        .serial(),
        run: Box::new(move |args, _ctx| {
            let agent = text_arg(args, "agent")?;
            let prompt = text_arg(args, "prompt")?;
            let custom_ws = optional_text(args, "workspace")?;
            let target_ws = match custom_ws {
                Some(p) => PathBuf::from(p),
                None => default_ws.clone(),
            };

            let mgr = delegate_mgr.clone();
            let summary = tauri::async_runtime::block_on(async move {
                mgr.delegate(&agent, &prompt, &target_ws).await
            })?;

            Ok(json!({
                "taskId": summary.task_id,
                "agent": summary.agent,
                "status": summary.status,
                "workspace": summary.workspace,
                "message": format!("已成功向 {} 委派任务（编号：{}）。请向用户说明已开始执行，并可在需要时调用 agent_status 查看执行进展。", summary.agent, summary.task_id)
            }).to_string())
        }),
    };

    let status_mgr = manager.clone();
    let status_tool = SimpleTool {
        spec: ToolSpec::new(
            "agent_status",
            "查询已委派外部 Agent 任务的当前执行状态、工具调用进展、最近输出或完成汇报。task_id: 委派任务编号。",
            json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "委派任务的编号（例如 del-1730000000-1）"
                    }
                },
                "required": ["task_id"]
            }),
        )
        .risk(Risk::Read)
        .parallel(),
        run: Box::new(move |args, _ctx| {
            let task_id = text_arg(args, "task_id")?;
            let summary = status_mgr.status(&task_id)
                .ok_or_else(|| format!("未找到编号为 {task_id} 的委派任务"))?;

            Ok(json!({
                "taskId": summary.task_id,
                "agent": summary.agent,
                "status": summary.status,
                "recentOutput": summary.recent_output,
                "pendingApproval": summary.pending_approval,
                "sessionId": summary.session_id,
            }).to_string())
        }),
    };

    let approve_mgr = manager.clone();
    let approve_tool = SimpleTool {
        spec: ToolSpec::new(
            "agent_approve",
            "为后台外部 Agent 的待确认操作提交允许或拒绝决定。task_id: 委派任务编号；approval_id: 待审批编号；decision: allow（允许）或 deny（拒绝）。",
            json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "委派任务的编号"
                    },
                    "approval_id": {
                        "type": "string",
                        "description": "待审批的操作编号"
                    },
                    "decision": {
                        "type": "string",
                        "enum": ["allow", "deny"],
                        "description": "审批决定：allow 允许，deny 拒绝"
                    }
                },
                "required": ["task_id", "approval_id", "decision"]
            }),
        )
        .risk(Risk::Write)
        .self_managed()
        .serial(),
        run: Box::new(move |args, _ctx| {
            let task_id = text_arg(args, "task_id")?;
            let approval_id = text_arg(args, "approval_id")?;
            let decision = text_arg(args, "decision")?;
            approve_mgr.approve(&task_id, &approval_id, &decision)?;
            Ok(format!("已成功为任务 {task_id} 提交审批决定：{decision}"))
        }),
    };

    let cancel_mgr = manager;
    let cancel_tool = SimpleTool {
        spec: ToolSpec::new(
            "agent_cancel",
            "停止或打断正在后台运行的外部 Agent 委派任务。task_id: 委派任务编号。",
            json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "委派任务的编号"
                    }
                },
                "required": ["task_id"]
            }),
        )
        .risk(Risk::Write)
        .self_managed()
        .serial(),
        run: Box::new(move |args, _ctx| {
            let task_id = text_arg(args, "task_id")?;
            cancel_mgr.cancel(&task_id)?;
            Ok(format!("已成功取消并打断委派任务 {task_id}"))
        }),
    };

    vec![
        Arc::new(delegate_tool),
        Arc::new(status_tool),
        Arc::new(approve_tool),
        Arc::new(cancel_tool),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_tools_have_expected_names_and_specs() {
        let mgr = Arc::new(DelegationManager::new());
        let list = tools(mgr, Path::new("."));
        let names: Vec<_> = list.iter().map(|t| t.spec().name.as_str()).collect();
        assert!(names.contains(&"agent_delegate"));
        assert!(names.contains(&"agent_status"));
        assert!(names.contains(&"agent_approve"));
        assert!(names.contains(&"agent_cancel"));
    }

    #[test]
    fn status_returns_error_for_unknown_task() {
        let mgr = Arc::new(DelegationManager::new());
        let list = tools(mgr, Path::new("."));
        let status_tool = list.iter().find(|t| t.spec().name == "agent_status").unwrap();
        let ctx = ToolCtx::detached("test-session");
        let res = status_tool.execute(json!({"task_id": "nonexistent"}), &ctx);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("未找到编号为 nonexistent"));
    }
}
