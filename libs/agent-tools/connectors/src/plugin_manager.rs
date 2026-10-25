use super::{
    plugin::{ConnectorPlugin, Transport},
    vault::{self, Vault},
    ConnectorManager,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, time::Duration};

/// Override values in the persisted state are vault field references, never credentials.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectorState {
    pub enabled: bool,
    pub user_disabled: bool,
    #[serde(default)]
    pub header_overrides: BTreeMap<String, String>,
    #[serde(default)]
    pub env_overrides: BTreeMap<String, String>,
}
impl Default for ConnectorState {
    fn default() -> Self {
        Self {
            enabled: true,
            user_disabled: false,
            header_overrides: BTreeMap::new(),
            env_overrides: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Binding {
    pub plugin: ConnectorPlugin,
    pub server: String,
}
pub(crate) fn load_states(root: &Path) -> Result<BTreeMap<String, ConnectorState>, String> {
    let path = root.join("connector-states.json");
    vault::regular(&path)?;
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let data = fs::read(path).map_err(|_| "无法读取连接器状态")?;
    if data.len() > 512 * 1024 {
        return Err("连接器状态过大".into());
    }
    serde_json::from_slice(&data).map_err(|_| "连接器状态损坏".into())
}
impl ConnectorManager {
    fn plugin_vault(&self) -> Result<Vault, String> {
        Vault::open(self.root.join("vault"))
    }
    pub fn plugin_state(&self, id: &str) -> ConnectorState {
        self.plugin_states
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(id)
            .cloned()
            .unwrap_or_default()
    }
    fn store_plugin_state(&self, id: &str, state: ConnectorState) -> Result<(), String> {
        let mut states = self.plugin_states.lock().unwrap_or_else(|e| e.into_inner());
        let mut updated = states.clone();
        updated.insert(id.into(), state);
        vault::atomic_write(
            &self.root.join("connector-states.json"),
            &serde_json::to_vec(&updated).map_err(|_| "无法保存连接器状态")?,
        )?;
        *states = updated;
        Ok(())
    }
    pub(crate) fn enable_plugin_state(&self, id: &str) -> Result<(), String> {
        let mut state = self.plugin_state(id);
        state.enabled = true;
        state.user_disabled = false;
        self.store_plugin_state(id, state)
    }
    pub(crate) fn disable_plugin_state(&self, id: &str) -> Result<(), String> {
        let mut state = self.plugin_state(id);
        state.enabled = false;
        state.user_disabled = true;
        self.store_plugin_state(id, state)
    }
    /// Credentials are retained on disconnect. Unbind explicitly removes them.
    pub fn unbind_plugin(&self, id: &str) -> Result<(), String> {
        let _operation = self
            .plugin_operations
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if !super::valid_id(id) {
            return Err("插件连接标识无效".into());
        }
        self.cancel_for_connector(id);
        let record = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .records
            .iter()
            .find(|r| r.id == id)
            .cloned();
        if let Some(record) = &record {
            if record.config.get("connectorPlugin").is_none() {
                return Err("该连接器不是插件连接器".into());
            }
            self.set_plugin_enabled_inner(id, false)?;
        }
        let session = record
            .as_ref()
            .and_then(|r| r.config["oauthSession"].as_str().map(str::to_owned));
        if let Some(session) = session {
            super::oauth::retire_credentials(&super::oauth::token_path(&self.root, id, &session)?);
        }
        let vault = self.plugin_vault()?;
        vault.remove_connector(id)?;
        vault.remove_connector(&format!("{id}/overrides"))?;
        self.store_plugin_state(
            id,
            ConnectorState {
                enabled: false,
                user_disabled: true,
                ..Default::default()
            },
        )
    }
    pub fn configured_plugin_fields(&self, id: &str) -> Result<Vec<String>, String> {
        self.plugin_vault()?.has(id)
    }
    pub fn connect_plugin(
        &self,
        plugin: &ConnectorPlugin,
        server: &str,
        values: &BTreeMap<String, String>,
        confirmed: bool,
    ) -> Result<Value, String> {
        if !confirmed {
            return Err("连接插件前需要用户明确确认配置".into());
        }
        let _operation = self
            .plugin_operations
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let spec = plugin.server(server)?;
        if let Some(reason) = spec.unsupported() {
            return Err(reason.into());
        }
        spec.validate_values(values, false)?;
        let id = plugin.connection_id(server);
        let vault = self.plugin_vault()?;
        let mut merged = self.plugin_values(plugin, server, &id, &vault)?;
        merged.extend(values.clone());
        plugin.build_config(server, &merged)?; // Validate before changing either file.
        for (key, value) in values {
            vault.set(&id, key, value)?;
        }
        let mut state = self.plugin_state(&id);
        state.enabled = true;
        state.user_disabled = false;
        self.store_plugin_state(&id, state)?;
        self.connect(
            &id,
            &plugin.display.name.chars().take(100).collect::<String>(),
            json!({"connectorPlugin":Binding { plugin: plugin.clone(), server: server.into() }}),
            true,
        )
    }
    /// OAuth discovery reuses the existing browser flow; only references enter connections.json.
    pub fn begin_connect_plugin(
        self: &std::sync::Arc<Self>,
        plugin: &ConnectorPlugin,
        server: &str,
        values: &BTreeMap<String, String>,
        confirmed: bool,
    ) -> Result<Value, String> {
        match self.connect_plugin(plugin, server, values, confirmed) {
            Ok(mut status) => {
                status["status"] = json!("connected");
                Ok(status)
            }
            Err(error)
                if confirmed
                    && (error.starts_with("MCP HTTP 401")
                        || error.contains(super::oauth::AUTH_REQUIRED)) =>
            {
                let id = plugin.connection_id(server);
                self.begin_connect(&id, &plugin.display.name.chars().take(100).collect::<String>(), json!({"connectorPlugin":Binding { plugin: plugin.clone(), server: server.into() }}), true)
            }
            Err(error) => Err(error),
        }
    }
    fn plugin_values(
        &self,
        plugin: &ConnectorPlugin,
        server: &str,
        id: &str,
        vault: &Vault,
    ) -> Result<BTreeMap<String, String>, String> {
        let mut values = BTreeMap::new();
        if let Some(schema) = plugin.server(server)?.schema() {
            for field in &schema.fields {
                if let Some(value) = vault.get(id, &field.key)? {
                    values.insert(field.key.clone(), value);
                }
            }
        }
        Ok(values)
    }
    pub(crate) fn resolve_plugin_config(
        &self,
        id: &str,
        config: &Value,
    ) -> Result<(Value, Vec<String>, Duration), String> {
        let Some(binding) = config.get("connectorPlugin") else {
            return Ok((config.clone(), vec![], Duration::from_secs(45)));
        };
        let binding: Binding =
            serde_json::from_value(binding.clone()).map_err(|_| "插件连接配置无效")?;
        if binding.plugin.connection_id(&binding.server) != id {
            return Err("插件连接标识不匹配".into());
        }
        let state = self.plugin_state(id);
        if !state.enabled || state.user_disabled {
            return Err("连接器已停用".into());
        }
        let vault = self.plugin_vault()?;
        let values = self.plugin_values(&binding.plugin, &binding.server, id, &vault)?;
        let mut built = binding.plugin.build_config(&binding.server, &values)?;
        let spec = binding.plugin.server(&binding.server)?;
        for (target, overrides) in [
            ("headers", &state.header_overrides),
            ("env", &state.env_overrides),
        ] {
            if !overrides.is_empty() {
                let map = built[target]
                    .as_object_mut()
                    .ok_or("连接器覆盖项与传输不匹配")?;
                for (key, reference) in overrides {
                    if reference != &format!("{target}:{key}") {
                        return Err("连接器覆盖项引用无效".into());
                    }
                    let value = vault
                        .get(&format!("{id}/overrides"), reference)?
                        .ok_or("连接器覆盖项凭据缺失")?;
                    if target == "headers" {
                        map.retain(|k, _| !k.eq_ignore_ascii_case(key));
                    }
                    map.insert(key.clone(), json!(value));
                }
            }
        }
        if let Some(session) = config.get("oauthSession") {
            built["oauthSession"] = session.clone();
            built["authMode"] = json!("oauth");
        }
        Ok((
            built,
            spec.disabled_tools.clone(),
            Duration::from_millis(spec.timeout_ms.unwrap_or(45_000)),
        ))
    }
    pub fn set_plugin_enabled(&self, id: &str, enabled: bool) -> Result<(), String> {
        let _operation = self
            .plugin_operations
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        self.set_plugin_enabled_inner(id, enabled)
    }
    fn set_plugin_enabled_inner(&self, id: &str, enabled: bool) -> Result<(), String> {
        {
            let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if !state
                .records
                .iter()
                .any(|r| r.id == id && r.config.get("connectorPlugin").is_some())
            {
                return Err("插件连接器未配置".into());
            }
        }
        let mut state = self.plugin_state(id);
        state.enabled = enabled;
        state.user_disabled = !enabled;
        self.store_plugin_state(id, state)?;
        if enabled {
            self.reconnect(id).map(|_| ())
        } else {
            self.disconnect(id)
        }
    }
    /// Store overrides encrypted, revoke current tools, and require an explicit reconnect.
    pub fn set_plugin_overrides(
        &self,
        id: &str,
        headers: &BTreeMap<String, String>,
        env: &BTreeMap<String, String>,
    ) -> Result<(), String> {
        let _operation = self
            .plugin_operations
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let record = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .records
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or("插件连接器未配置")?;
        let binding: Binding = serde_json::from_value(record.config["connectorPlugin"].clone())
            .map_err(|_| "插件连接器未配置")?;
        let spec = binding.plugin.server(&binding.server)?;
        if (spec.transport == Transport::Stdio && !headers.is_empty())
            || (spec.transport != Transport::Stdio && !env.is_empty())
        {
            return Err("连接器覆盖项与传输不匹配".into());
        }
        for map in [headers, env] {
            super::string_map(Some(&json!(map)))?;
            if map.values().any(|v| v.chars().any(char::is_control)) {
                return Err("覆盖项不可含控制字符".into());
            }
        }
        let vault = self.plugin_vault()?;
        let mut state = self.plugin_state(id);
        state.header_overrides.clear();
        state.env_overrides.clear();
        for (target, input, output) in [
            ("headers", headers, &mut state.header_overrides),
            ("env", env, &mut state.env_overrides),
        ] {
            for (key, value) in input {
                let reference = format!("{target}:{key}");
                vault.set(&format!("{id}/overrides"), &reference, value)?;
                output.insert(key.clone(), reference);
            }
        }
        self.store_plugin_state(id, state)?;
        self.disconnect(id)
    }
}
