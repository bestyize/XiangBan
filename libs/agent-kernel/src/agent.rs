use crate::error::HarnessError;
use crate::middleware::Middleware;
use crate::provider::{ModelProvider, ToolSchema};
use crate::store::SessionStore;
use crate::tool::Tool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_retries: 3, base_delay: Duration::from_millis(500), max_delay: Duration::from_secs(8) }
    }
}

pub(crate) type PromptSection = Box<dyn Fn() -> String + Send + Sync>;

pub struct Agent {
    pub(crate) provider: Arc<dyn ModelProvider>,
    pub(crate) store: Arc<dyn SessionStore>,
    pub(crate) tools: Vec<Arc<dyn Tool>>,
    pub(crate) tool_index: HashMap<String, usize>,
    pub(crate) validators: Vec<jsonschema::Validator>,
    pub(crate) middleware: Vec<Arc<dyn Middleware>>,
    pub(crate) system_prompt: String,
    pub(crate) prompt_sections: Vec<PromptSection>,
    pub(crate) max_turns: Option<usize>,
    pub(crate) max_truncated_tool_retries: usize,
    pub(crate) retry: RetryPolicy,
    pub(crate) running: Mutex<HashSet<String>>,
    discovery: Option<Arc<crate::discovery::ToolDiscovery>>,
}

impl Agent {
    /// Add a host policy before existing middleware, before starting this agent.
    pub fn with_middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middleware.insert(0, Arc::new(middleware));
        self
    }

    pub fn builder(provider: Arc<dyn ModelProvider>, store: Arc<dyn SessionStore>) -> AgentBuilder {
        AgentBuilder {
            provider,
            store,
            tools: Vec::new(),
            deferred: HashSet::new(),
            auto_load: HashSet::new(),
            middleware: Vec::new(),
            system_prompt: String::new(),
            prompt_sections: Vec::new(),
            max_turns: Some(25),
            max_truncated_tool_retries: 3,
            retry: RetryPolicy::default(),
        }
    }

    /// Static prompt plus every section, rebuilt before each model call.
    pub(crate) fn system_text(&self) -> String {
        let mut parts = vec![self.system_prompt.clone()];
        parts.extend(self.prompt_sections.iter().map(|section| section()));
        parts.retain(|p| !p.trim().is_empty());
        parts.join("\n\n")
    }

    /// Schemas in registration order, so the request prefix stays stable for prompt caching.
    pub fn tool_schemas(&self) -> Vec<ToolSchema> {
        self.tools
            .iter()
            .map(|t| {
                let spec = t.spec();
                ToolSchema { name: spec.name.clone(), description: spec.description.clone(), parameters: spec.schema.clone() }
            })
            .collect()
    }

    /// Initial model-visible schemas; the complete registry remains available via tool_schemas.
    pub fn initial_tool_schemas(&self) -> Vec<ToolSchema> {
        let mut schemas = self.tool_schemas();
        if let Some(discovery) = &self.discovery { discovery.initial_schemas(&mut schemas); }
        schemas
    }

    /// Compact deferred-tool directory, included before context budgeting on each model call.
    pub fn tool_catalog(&self) -> String {
        self.discovery.as_ref().map_or_else(String::new, |d| d.catalog())
    }
}

pub struct AgentBuilder {
    provider: Arc<dyn ModelProvider>,
    store: Arc<dyn SessionStore>,
    tools: Vec<Arc<dyn Tool>>,
    deferred: HashSet<String>,
    auto_load: HashSet<String>,
    middleware: Vec<Arc<dyn Middleware>>,
    system_prompt: String,
    prompt_sections: Vec<PromptSection>,
    max_turns: Option<usize>,
    max_truncated_tool_retries: usize,
    retry: RetryPolicy,
}

impl AgentBuilder {
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Appended to the system prompt in registration order. Called before every model call, so
    /// content such as a skill index can change while a run is in progress.
    pub fn prompt_section(mut self, section: impl Fn() -> String + Send + Sync + 'static) -> Self {
        self.prompt_sections.push(Box::new(section));
        self
    }

    pub fn tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.push(Arc::new(tool));
        self
    }

    pub fn tool_arc(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Register normally for validation, permissions, execution and cleanup, but expose the
    /// full schema only after tool_search/tool_load. Does not install or connect anything.
    pub fn deferred_tool_arc(mut self, tool: Arc<dyn Tool>) -> Self {
        self.deferred.insert(tool.spec().name.clone());
        self.tools.push(tool);
        self
    }

    /// Like [`deferred_tool_arc`](Self::deferred_tool_arc), for first-party tools: a call that already
    /// passed the real schema is loaded on the spot instead of being refused until `tool_load`.
    /// Permission middleware still decides whether it runs. Third-party tools (MCP) keep the strict path.
    pub fn deferred_trusted_tool_arc(mut self, tool: Arc<dyn Tool>) -> Self {
        self.auto_load.insert(tool.spec().name.clone());
        self.deferred_tool_arc(tool)
    }

    pub fn middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub fn max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = Some(max_turns);
        self
    }

    /// Let productive work continue until completion, cancellation or a host budget.
    /// Finite max_turns remains the default for existing embedders.
    pub fn unlimited_turns(mut self) -> Self {
        self.max_turns = None;
        self
    }

    /// Recoveries after tool-call arguments hit the output limit without new successful tool
    /// progress. Zero pauses on the first cutoff. Completed calls are never replayed.
    pub fn max_truncated_tool_retries(mut self, retries: usize) -> Self {
        self.max_truncated_tool_retries = retries;
        self
    }

    pub fn retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    pub fn build(mut self) -> Result<Agent, HarnessError> {
        let discovery = if self.deferred.is_empty() { None } else {
            let schemas = self.tools.iter().filter(|t| self.deferred.contains(&t.spec().name)).map(|t| {
                let s = t.spec();
                ToolSchema { name: s.name.clone(), description: s.description.clone(), parameters: s.schema.clone() }
            }).collect();
            let discovery = Arc::new(crate::discovery::ToolDiscovery::new(schemas, std::mem::take(&mut self.auto_load)));
            self.tools.extend(discovery.tools());
            // Filter and add the directory before hooks, compaction and ContextTrim.
            self.middleware.insert(0, discovery.clone());
            Some(discovery)
        };
        let mut tool_index = HashMap::new();
        let mut validators = Vec::with_capacity(self.tools.len());
        for (i, tool) in self.tools.iter().enumerate() {
            let name = &tool.spec().name;
            let valid = !name.is_empty()
                && name.len() <= 64
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
            if !valid {
                return Err(HarnessError::Config(format!("invalid tool name `{name}`")));
            }
            if tool_index.insert(name.clone(), i).is_some() {
                return Err(HarnessError::Config(format!("duplicate tool name `{name}`")));
            }
            let validator = jsonschema::options()
                .with_retriever(NoExternalSchemas)
                .build(&tool.spec().schema)
                .map_err(|e| HarnessError::Config(format!("invalid schema for tool `{name}`: {e}")))?;
            validators.push(validator);
        }
        if self.max_turns == Some(0) {
            return Err(HarnessError::Config("max_turns must be at least 1".into()));
        }
        Ok(Agent {
            provider: self.provider,
            store: self.store,
            tools: self.tools,
            tool_index,
            validators,
            middleware: self.middleware,
            system_prompt: self.system_prompt,
            prompt_sections: self.prompt_sections,
            max_turns: self.max_turns,
            max_truncated_tool_retries: self.max_truncated_tool_retries,
            retry: self.retry,
            running: Mutex::new(HashSet::new()),
            discovery,
        })
    }
}

/// Tool schemas may refer to definitions inside themselves, never to host files or the network.
struct NoExternalSchemas;

impl jsonschema::Retrieve for NoExternalSchemas {
    fn retrieve(&self, _uri: &jsonschema::Uri<String>) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        Err("external tool schema references are disabled; use local $defs/$ref".into())
    }
}
