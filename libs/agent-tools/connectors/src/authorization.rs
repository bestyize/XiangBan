use super::{normalized_config, oauth, ConnectorManager};
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub(super) struct Pending {
    pub id: String,
    pub cancel: AtomicBool,
    url: String,
    session_id: String,
    created: Instant,
    status: Mutex<Value>,
}
impl ConnectorManager {
    /// Connect public/key-based services immediately; launch standard OAuth only on an auth challenge.
    pub fn begin_connect(
        self: &Arc<Self>,
        id: &str,
        name: &str,
        config: Value,
        confirmed: bool,
    ) -> Result<Value, String> {
        if !confirmed {
            return Err("连接前需要确认服务和配置".into());
        }
        let config = normalized_config(&config)?;
        let (effective, _, _) = self.resolve_plugin_config(id, &config)?;
        self.cancel_for_connector(id);
        let error = match self.connect(id, name, config.clone(), true) {
            Ok(mut status) => {
                status["status"] = json!("connected");
                return Ok(status);
            }
            Err(e) => e,
        };
        let mode = effective["authMode"].as_str().unwrap_or("auto");
        let explicit_key = effective["headers"]
            .as_object()
            .is_some_and(|m| m.keys().any(|k| k.eq_ignore_ascii_case("authorization")));
        if effective["url"].as_str().is_none()
            || !matches!(mode, "auto" | "oauth")
            || (mode == "auto" && explicit_key)
            || !(error.starts_with("MCP HTTP 401") || error.contains(oauth::AUTH_REQUIRED))
        {
            return Err(error);
        }
        let mut flow = oauth::Flow::begin(&effective, &self.root, id)?;
        flow.encrypt_credentials = config.get("connectorPlugin").is_some();
        let flow_id = oauth::random()?;
        let pending = Arc::new(Pending {
            id: id.into(),
            cancel: AtomicBool::new(false),
            url: flow.url.clone(),
            session_id: flow.session_id.clone(),
            created: Instant::now(),
            status: Mutex::new(json!({"status":"authorizing"})),
        });
        {
            let mut flows = self
                .authorizations
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            flows.retain(|_, p| p.created.elapsed() < Duration::from_secs(360));
            if flows.len() >= 16 {
                return Err("等待授权的连接过多，请取消后重试".into());
            }
            for previous in flows.values().filter(|p| p.id == id) {
                previous.cancel.store(true, Ordering::SeqCst);
            }
            flows.insert(flow_id.clone(), pending.clone());
        }
        let manager = self.clone();
        let id = id.to_string();
        let name = name.to_string();
        let mut config = config;
        let task = pending.clone();
        std::thread::Builder::new()
            .name("mcp-oauth".into())
            .spawn(move || {
                let sid = flow.session_id.clone();
                let result = flow.complete(&task.cancel);
                let mut status = task.status.lock().unwrap_or_else(|e| e.into_inner());
                match result {
                    Ok(session) => {
                        if task.cancel.load(Ordering::SeqCst)
                            || manager.shutting_down.load(Ordering::SeqCst)
                        {
                            session.retire();
                            *status = json!({"status":"cancelled"});
                            return;
                        }
                        config["authMode"] = json!("oauth");
                        config["oauthSession"] = json!(sid);
                        match manager.connect(&id, &name, config, true) {
                            Ok(connection) => {
                                if task.cancel.load(Ordering::SeqCst)
                                    || manager.shutting_down.load(Ordering::SeqCst)
                                {
                                    let _ = manager.disconnect_oauth_generation(&id, &sid);
                                    session.retire();
                                    *status = json!({"status":"cancelled"});
                                } else {
                                    *status = json!({"status":"connected","connection":connection});
                                }
                            }
                            Err(error) => {
                                session.retire();
                                *status = json!({"status":"error","error":error});
                            }
                        }
                    }
                    Err(error) => {
                        if let Ok(path) = oauth::token_path(&manager.root, &id, &sid) {
                            oauth::retire_credentials(&path);
                        }
                        *status = if task.cancel.load(Ordering::SeqCst) {
                            json!({"status":"cancelled"})
                        } else {
                            json!({"status":"error","error":error})
                        };
                    }
                }
            })
            .map_err(|_| {
                pending.cancel.store(true, Ordering::SeqCst);
                "无法启动网页授权"
            })?;
        Ok(
            json!({"status":"authorization_required","flowId":flow_id,"authorizationUrl":pending.url}),
        )
    }
    pub fn reconnect_with_authorization(self: &Arc<Self>, id: &str) -> Result<Value, String> {
        let record = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .records
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or("连接器未配置")?;
        if record.config.get("connectorPlugin").is_some() { self.enable_plugin_state(id)?; }
        self.begin_connect(id, &record.name, record.config, true)
    }
    pub fn authorization_status(&self, flow: &str) -> Result<Value, String> {
        let pending = self
            .authorizations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(flow)
            .cloned()
            .ok_or("授权会话已结束，请重新连接")?;
        let status = match pending.status.try_lock() {
            Ok(v) => v.clone(),
            Err(_) => json!({"status":"connecting"}),
        };
        Ok(status)
    }
    pub fn cancel_authorization(&self, flow: &str) -> Result<(), String> {
        let pending = self
            .authorizations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(flow)
            .cloned();
        if let Some(pending) = pending {
            pending.cancel.store(true, Ordering::SeqCst);
            let mut status = pending.status.lock().unwrap_or_else(|e| e.into_inner());
            if status["status"] == "connected" {
                self.disconnect_oauth_generation(&pending.id, &pending.session_id)?;
            }
            *status = json!({"status":"cancelled"});
        }
        Ok(())
    }
    pub(super) fn cancel_for_connector(&self, id: &str) {
        for pending in self
            .authorizations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .filter(|p| p.id == id)
        {
            pending.cancel.store(true, Ordering::SeqCst);
        }
    }
    fn disconnect_oauth_generation(&self, id: &str, session_id: &str) -> Result<(), String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let mut records = state.records.clone();
        let Some(record) = records
            .iter_mut()
            .find(|r| r.id == id && r.config["oauthSession"] == session_id)
        else {
            return Ok(());
        };
        record.enabled = false;
        let saved = self.save(&records);
        if let Some(active) = state.active.remove(id) {
            active.close();
            if let Some(session) = &active.oauth {
                session.retire();
            }
        }
        state.records = records;
        saved
    }
}
