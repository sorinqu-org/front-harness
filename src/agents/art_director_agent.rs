use crate::agents::base_agent::BaseAgent;
use crate::config::constants::SYSTEM_PROMPT_ART_DIRECTOR;
use crate::core::event_bus::EventBus;
use crate::llm::provider::LlmProvider;
use crate::skills::registry::SkillRegistry;
use crate::tools::base::ToolRegistry;
use anyhow::Result;
use std::sync::Arc;

pub struct ArtDirectorAgent {
    agent: BaseAgent,
}

impl ArtDirectorAgent {
    pub fn new(llm: Arc<LlmProvider>, tools: ToolRegistry, event_bus: Option<EventBus>) -> Self {
        let system_prompt = SkillRegistry::get_combined_system_prompt(SYSTEM_PROMPT_ART_DIRECTOR);
        let agent = BaseAgent::new("ArtDirectorAgent", &system_prompt, llm, tools, event_bus);
        Self { agent }
    }

    pub async fn create_design_system(
        &self,
        research_brief: &str,
        audit_data: &str,
        business_goal: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Design a modern, high-conversion, accessible frontend architecture.\n\nBusiness Goal: {}\n\nAudit Data:\n{}\n\nResearch Brief:\n{}\n\nDeliverables:\n1. Macrostructure choice (e.g. Bento Grid / Workbench / Split Screen / Marquee Hero)\n2. Strict Color Palette (Primary Neutral, Accent Color with hex/oklch, Background)\n3. Typography Hierarchy (Headings font, Body font, size scale)\n4. Motion Directives (ScrollTrigger reveals, spring hover states)\n5. Component Composition & Navigation Architecture\n6. STRICT RULE: Zero Unicode emojis. Specify Lucide SVG icons.",
            business_goal, audit_data, research_brief
        );

        let spec = self.agent.run_turn(&prompt, &[]).await?;
        Ok(spec)
    }
}
