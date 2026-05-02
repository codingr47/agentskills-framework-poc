use std::collections::HashMap;

use crate::tooling::{
    spec::{LLMTool, LLMToolHandler, ToolHandlerArgument},
    tools::{read::Read, write::Write},
};

pub mod bash;
pub mod handlers;
pub mod read;
pub mod read_skill;
pub mod write;

fn boxed<F, Fut>(f: F) -> LLMToolHandler
where
    F: Fn(ToolHandlerArgument) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = serde_json::Value> + Send + 'static,
{
    Box::new(move |args| Box::pin(f(args)))
}

pub struct ToolsManager {
    pub tools: Vec<Box<dyn LLMTool>>,
    handlers: HashMap<String, LLMToolHandler>,
}

impl ToolsManager {
    pub fn new() -> Self {
        let mut instance = Self {
            tools: vec![
                Box::new(Read::new()),
                Box::new(Write::new()),
                Box::new(bash::Bash::new()),
                Box::new(read_skill::ReadSkill::new()),
            ],
            handlers: HashMap::new(),
        };

        instance.register_handler("Read".to_string(), boxed(handlers::read::read_handler));
        instance.register_handler("Write".to_string(), boxed(handlers::write::write_handler));
        instance.register_handler("Bash".to_string(), boxed(handlers::command::bash));
        instance.register_handler(
            "ReadSkill".to_string(),
            boxed(handlers::read_skill::read_skill_handler),
        );

        instance
    }

    pub fn json(&self) -> Vec<serde_json::Value> {
        self.tools.iter().map(|t| t.json()).collect()
    }

    pub async fn execute(
        &self,
        tool_name: String,
        arguments: ToolHandlerArgument,
    ) -> Option<serde_json::Value> {
        if let Some(handler) = self.handlers.get(tool_name.as_str()) {
            Some(handler(arguments).await)
        } else {
            None
        }
    }

    pub fn register_handler(&mut self, tool_name: String, handler: LLMToolHandler) {
        self.handlers.insert(tool_name, handler);
    }
}
