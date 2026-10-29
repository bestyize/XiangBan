//! Conversational connector discovery and user-operated installation/authorization.
use super::desktop_tools::Desktop;
use agent_kernel::{Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};

struct ConnectorTool { spec: ToolSpec, desktop: Arc<dyn Desktop> }
impl Tool for ConnectorTool {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn resolve_call(&self, args: &Value) -> Result<Option<(Arc<dyn Tool>, Value)>, String> {
        if self.spec.name != "connector_call" { return Ok(None); }
        let name = args["tool"].as_str().ok_or("缺少工具名称")?;
        Ok(self.desktop.connector_target(name)?.map(|tool| (tool, args["arguments"].clone())))
    }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() { return Err("连接器操作已取消".into()); }
        let value = match self.spec.name.as_str() {
            "connector_search" => self.desktop.connector_search(args["keyword"].as_str().unwrap_or("").trim())?,
            "connector_request" => {
                let refs = args["references"].as_array().ok_or("references 必须是数组")?;
                if refs.is_empty() || refs.len() > 3 { return Err("每次请选择 1 至 3 个连接器".into()); }
                let refs = refs.iter().map(|r| r.as_str().filter(|r| r.len() <= 256).map(String::from).ok_or("连接器引用无效"))
                    .collect::<Result<Vec<_>, _>>()?;
                self.desktop.connector_request(&refs, ctx)?
            }
            "connector_call" => {
                let result = self.execute_result(args, ctx);
                return if result.is_error { Err(result.output) } else { Ok(result.output) };
            }
            _ => return Err("未知连接器操作".into()),
        };
        Ok(value.to_string())
    }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        if self.spec.name == "connector_call" {
            if ctx.is_cancelled() { return ToolResult::error("连接器操作已取消"); }
            let Some(name) = args["tool"].as_str() else { return ToolResult::error("缺少工具名称"); };
            if !args["arguments"].is_object() { return ToolResult::error("工具参数必须是对象"); }
            return self.desktop.connector_call(name, args["arguments"].clone(), ctx);
        }
        match self.execute(args, ctx) { Ok(output) => ToolResult::ok(output), Err(error) => ToolResult::error(error) }
    }
}
pub(super) fn tools(desktop: Arc<dyn Desktop>) -> Vec<Arc<dyn Tool>> {
    let search = ToolSpec::new("connector_search", "搜索已安装连接器与小伴市场，最多返回三个候选及 reference。任务需要外部服务时先搜索，再调用 connector_request 展示连接卡片；不要让用户提供 MCP 地址或在聊天中发送密钥。空关键词列出已安装服务。", json!({"type":"object","properties":{"keyword":{"type":"string","maxLength":80}},"required":["keyword"]})).risk(Risk::Read).untrusted_output();
    let request = ToolSpec::new("connector_request", "用 connector_search 返回的 reference 展示连接卡片并等待用户安装、配置和网页登录。最多三个；用户完成连接后本工具返回可立即通过 connector_call 使用的工具及参数，不需要另开对话。用户跳过时不要反复推荐。", json!({"type":"object","properties":{"references":{"type":"array","items":{"type":"string"},"minItems":1,"maxItems":3}},"required":["references"]})).risk(Risk::Read).untrusted_output().timeout(Duration::from_secs(30 * 60));
    let mut call = ToolSpec::new("connector_call", "调用已连接服务的工具。tool 和 arguments 必须按 connector_request 返回的工具名称与 schema 填写。可用于本轮新连接的服务；若明确授权失效，重新调用 connector_request，超时或未知结果不能自动重放写操作。", json!({"type":"object","properties":{"tool":{"type":"string"},"arguments":{"type":"object"}},"required":["tool","arguments"]}));
    call.risk = Risk::Exec;
    call.untrusted_output = true;
    call.always_ask = true;
    vec![search, request, call].into_iter().map(|spec| Arc::new(ConnectorTool { spec, desktop: desktop.clone() }) as Arc<dyn Tool>).collect()
}
