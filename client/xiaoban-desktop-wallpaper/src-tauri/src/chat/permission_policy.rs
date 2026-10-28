//! Per-session permission policy shared by file access, commands and child agents.
use super::{danger, file_access};
use agent_kernel::{Permission, PermissionMode};
use std::{collections::HashMap, path::PathBuf, sync::{Arc, Mutex}};
fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

/// Conversation id → the folder it works in.
pub(super) type Workspaces = Arc<Mutex<HashMap<String, PathBuf>>>;

/// The folder a call works in: its conversation's, or for a research helper (`s1#research-…`) its parent's.
pub(super) fn folder_for(workspaces: &Workspaces, session: &str) -> Option<PathBuf> {
    let map = lock(workspaces);
    map.get(session).or_else(|| map.get(session.split('#').next().unwrap_or(session))).cloned()
}

/// "完全信任" runs everything except what `danger::assess` flags, which is asked every time.
/// Shared by the app and its tests so the tests exercise the rules that really apply.
pub(super) fn permission_with_danger_check(mode: PermissionMode, approver: impl agent_kernel::Approver + 'static, workspaces: Workspaces) -> Permission {
    Permission::new(mode, approver).danger_check_in_mode(move |mode, session, tool, args| {
        // In Auto mode, always_ask alone does not trigger approval when a danger
        // policy is installed; third-party skill installation must be flagged here.
        if tool == "phone_pack_install" {
            return Some("下载或更新第三方手机操作包需要确认来源及固定摘要；安装不会执行脚本或覆盖本机学习补丁".to_string());
        }
        if tool == "phone_workflow_run" {
            return Some("即将在真实手机连续执行已审查且摘要固定的应用工作流，需要核对全部步骤、目标应用及输入参数".to_string());
        }
        if tool == "skill_install" {
            return Some("安装或更新第三方 Skill 必须确认来源和内容摘要".to_string());
        }
        if tool == "care_set" && args["kind"] == "agent" {
            return Some("定时任务会在你不在时自动运行，添加或修改都需要你确认".to_string());
        }
        if mode != PermissionMode::Auto && folder_for(&workspaces, session).as_deref().is_some_and(|workspace| file_access::is_external_write(tool, args, workspace)) {
            return Some(file_access::EXTERNAL_WRITE_NOTICE.to_string());
        }
        if tool == "computer_use_repair_permissions" {
            return Some("清除 com.ifuryst.opencomputeruse 的所选旧授权（同标识独立安装版也受影响），之后需重新授权；每次都需确认".to_string());
        }
        danger::assess(tool, args, folder_for(&workspaces, session).as_deref()).map(str::to_string)
    })
    // "这类命令都允许" for one kind of simple command (`npm test`), never for commands in general.
    .allow_tool(open_claude_design::NAME)
    .remember_exec_as(danger::remember_key)
    .check_reads()
    .honor_grants_after_taint()
    // 专家团成员的会话（`<对话>#team-…`）沿用主对话的权限模式、授权和确认卡片。
    .children_inherit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::{Approval, ApprovalRequest, Middleware, Risk, ToolCall, ToolCtx, ToolGate, ToolSpec};
    use serde_json::json;

    #[test]
    fn trust_removes_location_limits_but_keeps_danger_review_and_session_boundaries() {
        let root = std::env::temp_dir().join(format!("permission-policy-{}", std::process::id()));
        let workspace = root.join("work");
        std::fs::create_dir_all(&workspace).unwrap();
        let workspaces = Arc::new(Mutex::new(HashMap::from([("trusted".into(), workspace.clone()), ("default".into(), workspace)])));
        let permission = permission_with_danger_check(PermissionMode::Ask, |_: &ApprovalRequest<'_>| Approval::Deny, workspaces);
        permission.set_session_mode("trusted", PermissionMode::Auto);
        let check = |session: &str, tool: &str, risk, args: serde_json::Value| {
            permission.before_tool(&ToolCtx::detached(session), &ToolCall { id: "c".into(), name: tool.into(), arguments: args.to_string() },
                &args, &ToolSpec::new(tool, "test", json!({"type":"object"})).risk(risk))
        };
        let outside = json!({"path":root.join("note.txt"),"content":"ordinary file"});
        assert!(matches!(check("trusted#team-worker", "write_file", Risk::Write, outside.clone()), ToolGate::Proceed));
        assert!(matches!(check("default", "write_file", Risk::Write, outside.clone()), ToolGate::Block(_)));
        for tool in ["shell", "process_start"] {
            for command in ["npm uninstall -g 9router", "brew install jq"] {
                assert!(matches!(check("trusted", tool, Risk::Exec, json!({"command":command})), ToolGate::Proceed), "{tool}: {command}");
            }
            for command in ["sudo true", "rm -rf /outside", "git push --force"] {
                assert!(matches!(check("trusted", tool, Risk::Exec, json!({"command":command})), ToolGate::Block(_)), "{tool}: {command}");
            }
        }
        assert!(matches!(check("trusted", "write_file", Risk::Write, json!({"path":root.join(".zshrc"),"content":"x"})), ToolGate::Block(_)));
        permission.set_session_mode("trusted", PermissionMode::Ask);
        assert!(matches!(check("trusted#team-worker", "write_file", Risk::Write, outside), ToolGate::Block(_)));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn full_trust_allows_reviewed_extensions_and_ordinary_input_but_keeps_sensitive_reads() {
        let root=std::env::temp_dir().join(format!("permission-policy-extensions-{}",std::process::id()));
        let work=root.join("work");let credential=root.join(".ssh/id_ed25519");
        std::fs::create_dir_all(&work).unwrap();std::fs::create_dir_all(credential.parent().unwrap()).unwrap();
        std::fs::write(&credential,"synthetic credential fixture").unwrap();
        let p=permission_with_danger_check(PermissionMode::Auto,|_:&ApprovalRequest<'_>|Approval::Deny,
            Arc::new(Mutex::new(HashMap::from([("s".into(),work.clone())]))));
        let check=|tool:&str,risk,args:serde_json::Value| p.before_tool(&ToolCtx::detached("s"),
            &ToolCall{id:"c".into(),name:tool.into(),arguments:args.to_string()},&args,
            &ToolSpec::new(tool,"test",json!({})).risk(risk).always_ask());
        for (tool,args) in [("skill_run",json!({"skill":"verified-skill","script":"scripts/test.py"})),
            ("mcp_ext_abc_query",json!({"query":"status"})),("process_write",json!({"id":"p","input":"yes\n"}))] {
            assert!(matches!(check(tool,Risk::Exec,args),ToolGate::Proceed),"{tool}");
        }
        assert!(matches!(check("skill_install",Risk::Write,json!({})),ToolGate::Block(_)));
        assert!(matches!(check("process_write",Risk::Exec,json!({"id":"p","input":"sudo true\n"})),ToolGate::Block(_)));
        for tool in ["read_file","list_dir","grep"] {
            assert!(matches!(check(tool,Risk::Read,json!({"path":credential})),ToolGate::Block(_)),"{tool}");
        }
        #[cfg(unix)]{
            std::os::unix::fs::symlink(&credential,work.join("alias.txt")).unwrap();
            assert!(matches!(check("read_file",Risk::Read,json!({"path":work.join("alias.txt")})),ToolGate::Block(_)));
        }
        assert!(matches!(check("read_file",Risk::Read,json!({"path":work})),ToolGate::Proceed));
        std::fs::remove_dir_all(root).unwrap();
    }

}
