//! Provider-neutral disclosure: load real function schemas, never proxy execution.
use crate::{HarnessError, Message, Middleware, Risk, RunEnd, Tool, ToolCall, ToolCtx, ToolGate, ToolSchema, ToolSpec, Turn};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

const CATALOG_BYTES: usize = 6000;
const NAME_BYTES: usize = 1500;

/// Used by installed-skill search too. Unicode substring tokens support Chinese descriptions
/// without a remote embedding dependency; exact names get priority.
pub(crate) fn relevance(query: &str, name: &str, description: &str) -> usize {
    let query = query.trim().to_lowercase();
    if query.is_empty() { return 0; }
    let name = name.to_lowercase();
    let description = description.to_lowercase();
    if name == query { return usize::MAX; }
    let mut terms = HashSet::new();
    for word in query.split(|c: char| !c.is_alphanumeric()).filter(|s| !s.is_empty()) {
        if word.chars().any(|c| ('\u{3400}'..='\u{9fff}').contains(&c)) {
            let chars: Vec<_> = word.chars().collect();
            if chars.len() == 1 { terms.insert(word.to_string()); }
            for pair in chars.windows(2) { terms.insert(pair.iter().collect::<String>()); }
        } else { terms.insert(word.to_string()); }
    }
    let score = terms.iter().map(|t| {
        let field_score = if name.contains(t.as_str()) { 6 } else if description.contains(t.as_str()) { 2 } else { return 0 };
        // Generic action words must not outweigh the service and object in a
        // multi-word query ("GitHub list repositories" versus "list apps").
        let weight = if matches!(t.as_str(), "list" | "get" | "find" | "search" | "show") { 10 } else { 100 };
        weight + field_score
    }).sum::<usize>();
    score + if name.contains(&query) || description.contains(&query) { 10 } else { 0 }
}

pub(crate) struct ToolDiscovery {
    schemas: Vec<ToolSchema>,
    names: HashSet<String>,
    // First-party tools whose first call loads them instead of being refused.
    auto_load: HashSet<String>,
    // A run owns one session; cleanup clears this on every terminal path.
    loaded: Mutex<HashMap<String, HashSet<String>>>,
}

impl ToolDiscovery {
    pub fn new(schemas: Vec<ToolSchema>, names: HashSet<String>, auto_load: HashSet<String>) -> Self {
        Self { schemas, names, auto_load, loaded: Mutex::new(HashMap::new()) }
    }

    pub fn initial_schemas(&self, schemas: &mut Vec<ToolSchema>) {
        schemas.retain(|s| !self.names.contains(&s.name));
    }

    pub fn catalog(&self) -> String {
        let mut out = String::from("## Additional tools\nThese tools are registered but their argument schemas are deferred. Before claiming a capability is unavailable, check this directory or call tool_search. Search covers tools registered with this agent; it does not search a store, install software or connect accounts. An empty search only means that query found no match. tool_search loads matching deferred schemas for the next model call. If you know the exact deferred name, use tool_load. Then call the real named tool normally; all existing permissions still apply. Loaded schemas remain available for this task. If the user explicitly requested a tool or delivery method and it is unavailable, explain the blockage and ask before substituting a different delivery method.\n");
        let mut unsummarized = Vec::new();
        for schema in self.schemas.iter().filter(|s| self.names.contains(&s.name)) {
            // Only a short, single-line summary, never full schemas or credentials.
            let summary: String = schema.description.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(72).collect();
            let line = format!("- {}: {}\n", schema.name, summary);
            if out.len() + line.len() > CATALOG_BYTES { unsummarized.push(schema.name.as_str()); continue; }
            out.push_str(&line);
        }
        if unsummarized.is_empty() { return out; }
        // A name costs a few bytes, so a tool whose summary does not fit can still be loaded by name.
        let mut names = String::new();
        let mut listed = 0;
        for name in &unsummarized {
            if names.len() + name.len() + 2 > NAME_BYTES { break; }
            if listed > 0 { names.push_str(", "); }
            names.push_str(name);
            listed += 1;
        }
        if listed > 0 { out.push_str(&format!("Summaries omitted for {listed} more tools: {names}.\n")); }
        if listed < unsummarized.len() {
            out.push_str(&format!("{} additional tools omitted from this directory; tool_search searches the full registered catalog.\n", unsummarized.len() - listed));
        }
        out
    }

    pub fn tools(self: &Arc<Self>) -> Vec<Arc<dyn Tool>> {
        vec![Arc::new(DiscoveryTool::new(self.clone(), false)), Arc::new(DiscoveryTool::new(self.clone(), true))]
    }

    fn load(&self, session: &str, names: &[String]) -> Result<String, String> {
        // Validate the entire batch before activating anything.
        if names.is_empty() || names.len() > 10 { return Err("provide 1–10 exact tool names".into()); }
        for name in names {
            if !self.names.contains(name) { return Err(format!("no deferred tool named `{name}`; use tool_search to find a registered name")); }
        }
        self.loaded.lock().unwrap_or_else(|e| e.into_inner()).entry(session.into()).or_default().extend(names.iter().cloned());
        Ok(json!({"loaded": names, "next_step": "The real argument schemas will be present in the next model request. Call those named tools directly; loading grants no execution permission."}).to_string())
    }
}

impl Middleware for ToolDiscovery {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        // A continuation builds a fresh agent. Restore disclosure from successful
        // historical calls; this restores schemas, never execution permission.
        let mut restored = HashSet::new();
        let results: HashMap<_, _> = turn.messages.iter().filter(|m| m.role == crate::Role::Tool && !m.is_error)
            .filter_map(|m| m.tool_call_id.as_deref().map(|id| (id, m))).collect();
        for message in &turn.messages {
            for call in &message.tool_calls {
                let Some(result) = results.get(call.id.as_str()) else { continue };
                if self.names.contains(&call.name) { restored.insert(call.name.clone()); }
                if matches!(call.name.as_str(), "tool_load" | "tool_search") {
                    if let Ok(value) = serde_json::from_str::<Value>(&result.text()) {
                        for name in value["loaded"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                            if self.names.contains(name) { restored.insert(name.to_string()); }
                        }
                    }
                }
            }
        }
        if !restored.is_empty() {
            self.loaded.lock().unwrap_or_else(|e| e.into_inner()).entry(turn.session.clone()).or_default().extend(restored);
        }
        let loaded = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
        let active = loaded.get(&turn.session);
        turn.tools.retain(|s| !self.names.contains(&s.name) || active.is_some_and(|names| names.contains(&s.name)));
        let catalog = self.catalog();
        // Append to the existing system message so it is budgeted like other instructions.
        if let Some(system) = turn.messages.iter_mut().find(|m| m.role == crate::Role::System) {
            system.parts.push(crate::Part::Text { text: format!("\n\n{catalog}") });
        } else { turn.messages.insert(0, Message::system(catalog)); }
        Ok(())
    }

    fn before_tool(&self, ctx: &ToolCtx, call: &ToolCall, _args: &Value, _spec: &ToolSpec) -> ToolGate {
        if self.names.contains(&call.name) && !self.loaded.lock().unwrap_or_else(|e| e.into_inner()).get(ctx.session()).is_some_and(|set| set.contains(&call.name)) {
            if self.auto_load.contains(&call.name) {
                // The arguments already passed this tool's real schema in `exec`, so refusing would only cost
                // a round trip (models repeated the refused call up to four times). Load it for the next request.
                self.loaded.lock().unwrap_or_else(|e| e.into_inner()).entry(ctx.session().into()).or_default().insert(call.name.clone());
                return ToolGate::Proceed;
            }
            return ToolGate::Block(format!("Tool `{}` is deferred. Use tool_load with this exact name or tool_search first, then call it using its loaded schema. No tool was executed.", call.name));
        }
        ToolGate::Proceed
    }
}

struct DiscoveryTool {
    spec: ToolSpec,
    discovery: Arc<ToolDiscovery>,
    exact: bool,
}

impl DiscoveryTool {
    fn new(discovery: Arc<ToolDiscovery>, exact: bool) -> Self {
        let (name, description, schema) = if exact {
            ("tool_load", "Load full schemas for 1–10 exact names from the Additional tools directory; then call the real tools directly. Does not execute, install or connect anything.", json!({"type":"object","properties":{"names":{"type":"array","items":{"type":"string"},"minItems":1,"maxItems":10}},"required":["names"],"additionalProperties":false}))
        } else {
            ("tool_search", "Find tools registered with this agent by capability, service or tool name and load matching deferred schemas. This is a local catalog, not an online store. Try concise keywords; an empty result is not proof that all capabilities are absent.", json!({"type":"object","properties":{"query":{"type":"string","minLength":1,"maxLength":200},"limit":{"type":"integer","minimum":1,"maximum":10}},"required":["query"],"additionalProperties":false}))
        };
        Self { spec: ToolSpec::new(name, description, schema).risk(Risk::Read).max_output(12_000), discovery, exact }
    }
}

impl Tool for DiscoveryTool {
    fn spec(&self) -> &ToolSpec { &self.spec }

    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() { return Err("cancelled".into()); }
        if self.exact {
            let names = args["names"].as_array().ok_or("names must be an array")?.iter().map(|n| n.as_str().map(str::to_string).ok_or("names must contain strings")).collect::<Result<Vec<_>, _>>()?;
            return self.discovery.load(ctx.session(), &names);
        }
        let query = args["query"].as_str().ok_or("query must be a string")?.trim();
        if query.is_empty() { return Err("query must not be blank".into()); }
        let limit = args["limit"].as_u64().unwrap_or(5).clamp(1, 10) as usize;
        let mut matches: Vec<_> = self.discovery.schemas.iter().filter_map(|s| {
            let score = relevance(query, &s.name, &s.description);
            (score > 0).then_some((score, s))
        }).collect();
        matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
        let total = matches.len();
        matches.truncate(limit);
        let names: Vec<_> = matches.iter().filter(|(_, s)| self.discovery.names.contains(&s.name)).map(|(_, s)| s.name.clone()).collect();
        if !names.is_empty() { self.discovery.load(ctx.session(), &names)?; }
        Ok(json!({"matches": matches.iter().map(|(_, s)| json!({"name":s.name,"description":s.description.chars().take(160).collect::<String>(), "schema": if self.discovery.names.contains(&s.name) { "deferred" } else { "already_available" }})).collect::<Vec<_>>(), "total":total,"loaded":names,"hint":"Matched schemas are available on the next model call. An empty result only means this query found no match among tools registered with this agent; try a shorter keyword, service name or exact name from the directory. If the user explicitly requested an unavailable tool or delivery method, explain the blockage and ask before using a substitute."}).to_string())
    }

    fn on_run_end(&self, end: &RunEnd) -> Result<(), String> {
        if !self.exact { self.discovery.loaded.lock().unwrap_or_else(|e| e.into_inner()).remove(&end.session_id); }
        Ok(())
    }
}
