use crate::agents::base_agent::BaseAgent;
use crate::config::constants::SYSTEM_PROMPT_RESEARCH;
use crate::core::event_bus::EventBus;
use crate::llm::provider::LlmProvider;
use crate::skills::registry::SkillRegistry;
use crate::tools::base::ToolRegistry;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

pub struct ResearchAgent {
    agent: BaseAgent,
}

impl ResearchAgent {
    pub fn new(llm: Arc<LlmProvider>, tools: ToolRegistry, event_bus: Option<EventBus>) -> Self {
        let system_prompt = SkillRegistry::get_combined_system_prompt(SYSTEM_PROMPT_RESEARCH);
        let agent = BaseAgent::new("ResearchAgent", &system_prompt, llm, tools, event_bus);
        Self { agent }
    }

    pub async fn conduct_research(&self, target_topic: &str, industry: &str) -> Result<String> {
        let search_query = format!("modern {} web design patterns typography microinteractions", target_topic);
        let search_res = self
            .agent
            .call_tool("web_search", json!({ "query": search_query }))
            .await?;

        let prompt = format!(
            "Analyze the target niche '{}' in the industry '{}'.\n\nSearch Context:\n{}\n\nProduce a concise research brief covering:\n1. Target Audience & Core Conversion Goals\n2. Key Visual & Interactive Expectations\n3. Modern Competitor Flaws to Avoid (Slop, Clutter)",
            target_topic, industry, search_res
        );

        let brief = self.agent.run_turn(&prompt, &[]).await?;
        Ok(brief)
    }
}
