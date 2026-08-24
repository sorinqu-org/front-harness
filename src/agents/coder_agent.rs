use crate::agents::base_agent::BaseAgent;
use crate::config::constants::SYSTEM_PROMPT_CODER;
use crate::core::event_bus::EventBus;
use crate::llm::provider::LlmProvider;
use crate::skills::registry::SkillRegistry;
use crate::skills::stop_slop::StopSlopValidator;
use crate::tools::base::ToolRegistry;
use anyhow::Result;
use std::sync::Arc;

pub struct CoderAgent {
    agent: BaseAgent,
}

impl CoderAgent {
    pub fn new(llm: Arc<LlmProvider>, tools: ToolRegistry, event_bus: Option<EventBus>) -> Self {
        let system_prompt = SkillRegistry::get_combined_system_prompt(SYSTEM_PROMPT_CODER);
        let agent = BaseAgent::new("CoderAgent", &system_prompt, llm, tools, event_bus);
        Self { agent }
    }

    pub fn new_with_skills(
        llm: Arc<LlmProvider>,
        tools: ToolRegistry,
        event_bus: Option<EventBus>,
        enabled_skills: &[String],
        design_style: &str,
        references: &[String],
    ) -> Self {
        let system_prompt = SkillRegistry::build_custom_system_prompt(
            SYSTEM_PROMPT_CODER,
            enabled_skills,
            design_style,
            references,
        );
        let agent = BaseAgent::new("CoderAgent", &system_prompt, llm, tools, event_bus);
        Self { agent }
    }

    pub async fn generate_frontend(
        &self,
        design_spec: &str,
        original_content: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Implement a complete, production-ready, single-file modern frontend index.html with embedded Tailwind CSS via CDN, modern Google Fonts, GSAP 3.12 + ScrollTrigger, and inline Lucide SVG vector icons.\n\nDesign System & Directives:\n{}\n\nOriginal Site Content & Structure:\n{}\n\nREQUIREMENTS:\n- Complete runnable HTML document with DOCTYPE, head, responsive meta, CSS styles, semantic tags, and interactive scripts.\n- Smooth scrolling (Lenis or CSS smooth scroll) and GSAP animations.\n- Fully responsive layout from 375px (mobile) to 1920px (desktop).\n- Clear, accessible conversion path: prominent phone call button, quick order modal/form, working interactive accordion or tabs.\n- ZERO UNICODE EMOJIS. Use clean vector SVG icons only.\n- Output ONLY the complete raw HTML code enclosed in ```html ... ``` code block.",
            design_spec, original_content
        );

        let generated_code = self.agent.run_turn(&prompt, &[]).await?;
        let extracted_html = extract_code_block(&generated_code, "html")
            .unwrap_or(generated_code);

        let warnings = StopSlopValidator::audit_code(&extracted_html);
        for w in warnings {
            if let Some(bus) = &self.agent.event_bus {
                bus.emit_log("warn", "CoderAgent", &w);
            }
        }

        Ok(extracted_html)
    }
}

fn extract_code_block(text: &str, language: &str) -> Option<String> {
    let start_tag = format!("```{}", language);
    if let Some(start_pos) = text.find(&start_tag) {
        let content_start = start_pos + start_tag.len();
        if let Some(end_pos) = text[content_start..].find("```") {
            return Some(text[content_start..content_start + end_pos].trim().to_string());
        }
    } else if let Some(start_pos) = text.find("```") {
        let content_start = start_pos + 3;
        let slice = &text[content_start..];
        let actual_start = if let Some(newline_pos) = slice.find('\n') {
            content_start + newline_pos + 1
        } else {
            content_start
        };
        if let Some(end_pos) = text[actual_start..].find("```") {
            return Some(text[actual_start..actual_start + end_pos].trim().to_string());
        }
    }
    None
}
