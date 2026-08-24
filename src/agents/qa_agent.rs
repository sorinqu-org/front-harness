use crate::agents::base_agent::BaseAgent;
use crate::config::constants::SYSTEM_PROMPT_QA;
use crate::core::event_bus::EventBus;
use crate::llm::provider::LlmProvider;
use crate::tools::base::ToolRegistry;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

pub struct QaAgent {
    agent: BaseAgent,
}

impl QaAgent {
    pub fn new(llm: Arc<LlmProvider>, tools: ToolRegistry, event_bus: Option<EventBus>) -> Self {
        let agent = BaseAgent::new("QaAgent", SYSTEM_PROMPT_QA, llm, tools, event_bus);
        Self { agent }
    }

    pub async fn verify_site(&self, local_url: &str) -> Result<String> {
        let audit_res = self
            .agent
            .call_tool("browser_audit", json!({ "url": local_url }))
            .await?;

        let prompt = format!(
            "Review the browser audit result of the locally generated site at {}:\n\n{}\n\nVerify:\n1. Layout rendering without visual overlaps\n2. Desktop and Mobile responsiveness\n3. Accessibility and contrast standards\n4. Confirm zero console errors\nProduce a concise QA pass summary.",
            local_url, audit_res
        );

        let qa_report = self.agent.run_turn(&prompt, &[]).await?;
        Ok(qa_report)
    }
}
