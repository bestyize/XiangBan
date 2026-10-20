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
    pub(crate) max_turns: usize,
    pub(crate) retry: RetryPolicy,
    pub(crate) running: Mutex<HashSet<String>>,
}

impl Agent {
    pub fn builder(provider: Arc<dyn ModelProvider>, store: Arc<dyn SessionStore>) -> AgentBuilder {
        AgentBuilder {
            provider,
            store,
            tools: Vec::new(),
            middleware: Vec::new(),
            system_prompt: String::new(),
            prompt_sections: Vec::new(),
            max_turns: 25,
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
}

pub struct AgentBuilder {
    provider: Arc<dyn ModelProvider>,
    store: Arc<dyn SessionStore>,
    tools: Vec<Arc<dyn Tool>>,
    middleware: Vec<Arc<dyn Middleware>>,
    system_prompt: String,
    prompt_sections: Vec<PromptSection>,
    max_turns: usize,
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

    pub fn middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub fn max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = max_turns;
        self
    }

    pub fn retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    pub fn build(self) -> Result<Agent, HarnessError> {
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
        if self.max_turns == 0 {
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
            retry: self.retry,
            running: Mutex::new(HashSet::new()),
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
