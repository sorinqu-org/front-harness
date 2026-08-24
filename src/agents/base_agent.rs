use crate::core::context_manager::{ChatMessage, ContextManager};
use crate::core::event_bus::EventBus;
use crate::llm::provider::LlmProvider;
use crate::tools::base::ToolRegistry;
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

pub struct BaseAgent {
    pub name: String,
    pub system_prompt: String,
    pub llm: Arc<LlmProvider>,
    pub tools: ToolRegistry,
    pub event_bus: Option<EventBus>,
}

impl BaseAgent {
    pub fn new(
        name: &str,
        system_prompt: &str,
        llm: Arc<LlmProvider>,
        tools: ToolRegistry,
        event_bus: Option<EventBus>,
    ) -> Self {
        Self {
            name: name.to_string(),
            system_prompt: system_prompt.to_string(),
            llm,
            tools,
            event_bus,
        }
    }

    pub async fn run_turn(&self, user_instruction: &str, context_history: &[ChatMessage]) -> Result<String> {
        let mut context = ContextManager::new(16384);
        context.set_system_prompt(&self.system_prompt);

        for msg in context_history {
            context.add_message(msg.clone());
        }
        context.add_message(ChatMessage::user(user_instruction));

        if let Some(bus) = &self.event_bus {
            bus.emit_log("info", &self.name, &format!("Processing instruction: {}", user_instruction.chars().take(80).collect::<String>()));
        }

        let response = self
            .llm
            .stream_chat(&self.name, context.messages(), self.event_bus.as_ref())
            .await?;

        Ok(response)
    }

    pub async fn call_tool(&self, tool_name: &str, input: Value) -> Result<String> {
        if let Some(bus) = &self.event_bus {
            bus.publish(crate::core::events::Event::ToolCallStart {
                agent: self.name.clone(),
                tool_name: tool_name.to_string(),
                input: input.to_string(),
            });
        }

        let result = if let Some(tool) = self.tools.get(tool_name) {
            let res = tool.execute(input).await?;
            if let Some(bus) = &self.event_bus {
                bus.publish(crate::core::events::Event::ToolCallEnd {
                    agent: self.name.clone(),
                    tool_name: tool_name.to_string(),
                    output: res.output.chars().take(500).collect(),
                    success: res.success,
                });
            }
            res.output
        } else {
            let err = format!("Tool '{}' not found", tool_name);
            if let Some(bus) = &self.event_bus {
                bus.emit_error(&self.name, &err);
            }
            err
        };

        Ok(result)
    }
}
