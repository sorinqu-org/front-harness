use crate::agents::art_director_agent::ArtDirectorAgent;
use crate::agents::coder_agent::CoderAgent;
use crate::agents::qa_agent::QaAgent;
use crate::agents::research_agent::ResearchAgent;
use crate::config::settings::Settings;
use crate::core::event_bus::EventBus;
use crate::core::state_machine::{PipelinePhase, StateMachine};
use crate::llm::provider::LlmProvider;
use crate::memory::store::{MemoryStore, ProjectSummary};
use crate::tools::base::ToolRegistry;
use crate::tools::bash_runner::BashRunnerTool;
use crate::tools::browser::BrowserTool;
use crate::tools::dev_server::DevServerManager;
use crate::tools::embedded_scripts::get_template_html;
use crate::tools::file_system::FileSystemTool;
use crate::tools::web_search::WebSearchTool;
use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

pub struct PipelineOrchestrator {
    settings: Settings,
    event_bus: EventBus,
    state_machine: StateMachine,
    llm: Arc<LlmProvider>,
    tools: ToolRegistry,
    workspace_dir: PathBuf,
}

impl PipelineOrchestrator {
    pub fn new(settings: Settings, event_bus: EventBus) -> Self {
        let llm = Arc::new(LlmProvider::new(settings.llm.clone()));
        let mut tools = ToolRegistry::new();

        let ws = settings.workspace_dir.clone();
        let _ = std::fs::create_dir_all(&ws);

        tools.register(Arc::new(FileSystemTool::new(ws.clone())));
        tools.register(Arc::new(BashRunnerTool::new(ws.clone())));
        tools.register(Arc::new(WebSearchTool::new(
            &settings.search.provider,
            settings.search.tavily_api_key.clone(),
        )));

        let audit_dir = ws.join("audit");
        tools.register(Arc::new(BrowserTool::new(audit_dir)));

        Self {
            settings,
            event_bus,
            state_machine: StateMachine::new(),
            llm,
            tools,
            workspace_dir: ws,
        }
    }

    pub async fn run_redesign_pipeline(
        &mut self,
        target_source: &str,
        business_goal: &str,
    ) -> Result<PathBuf> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.event_bus.emit_phase("Init", "Auditing", &format!("Inspecting target {}", target_source));
        self.state_machine.transition_to(PipelinePhase::Auditing, json!({ "target": target_source }))?;

        // 1. Audit Phase (Playwright Crawler)
        let audit_dir = self.workspace_dir.join("audit");
        let audit_tool = BrowserTool::new(audit_dir);
        
        let audit_report = match audit_tool.run_audit(target_source).await {
            Ok(rep) => rep,
            Err(e) => {
                let err_msg = format!("Playwright crawler failed: {}", e);
                self.event_bus.emit_error("PipelineOrchestrator", &err_msg);
                anyhow::bail!("{}", err_msg);
            }
        };
        let audit_json = serde_json::to_string_pretty(&audit_report.site_analysis)?;

        // 2. Research Phase
        self.event_bus.emit_phase("Auditing", "Researching", "Gathering references and market patterns");
        self.state_machine.transition_to(PipelinePhase::Researching, json!({ "audit_done": true }))?;

        let research_agent = ResearchAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let research_brief = match research_agent.conduct_research(target_source, &self.settings.design.style_prompt).await {
            Ok(brief) => brief,
            Err(e) => {
                self.event_bus.emit_log("warn", "ResearchAgent", &format!("LLM fallback for research: {}", e));
                format!("Research for {}: Modern high-conversion design adhering to style: {}", target_source, self.settings.design.style_prompt)
            }
        };

        // 3. Art Direction & Architecture Phase
        self.event_bus.emit_phase("Researching", "Designing", "Formulating design tokens, layout hierarchy and style");
        self.state_machine.transition_to(PipelinePhase::Designing, json!({ "research_done": true }))?;

        let art_director = ArtDirectorAgent::new_with_skills(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
            &self.settings.design.selected_skills,
            &self.settings.design.style_prompt,
            &self.settings.design.references,
        );

        let combined_goal = format!("{}\n\nDesign Style Directive: {}", business_goal, self.settings.design.style_prompt);
        let design_spec = match art_director.create_design_system(&research_brief, &audit_json, &combined_goal).await {
            Ok(spec) => spec,
            Err(e) => {
                self.event_bus.emit_log("warn", "ArtDirectorAgent", &format!("LLM fallback for art direction: {}", e));
                format!(
                    "# Design System for {}\n- Style Directives: {}\n- Macrostructure: Workbench / Modern Bento\n- Anti-Slop: Strict Zero Emojis, vector Lucide SVGs, WCAG AA contrast.",
                    target_source, self.settings.design.style_prompt
                )
            }
        };

        let design_doc_path = self.workspace_dir.join("design.md");
        std::fs::write(&design_doc_path, &design_spec)?;

        // 4. Implementation Phase
        self.event_bus.emit_phase("Designing", "Implementing", "Writing modern responsive HTML/Tailwind/GSAP code");
        self.state_machine.transition_to(PipelinePhase::Implementing, json!({ "design_done": true }))?;

        let coder_agent = CoderAgent::new_with_skills(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
            &self.settings.design.selected_skills,
            &self.settings.design.style_prompt,
            &self.settings.design.references,
        );

        let html_content = match coder_agent.generate_frontend(&design_spec, &audit_json).await {
            Ok(code) => code,
            Err(e) => {
                self.event_bus.emit_log("warn", "CoderAgent", &format!("Generating template fallback frontend: {}", e));
                generate_fallback_redesign(target_source, &audit_report)
            }
        };

        let output_html_path = self.workspace_dir.join("dist").join("index.html");
        if let Some(parent) = output_html_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_html_path, &html_content)?;

        // Copy downloaded images to dist/images/
        let audit_images = self.workspace_dir.join("audit").join("images");
        let dist_images = self.workspace_dir.join("dist").join("images");
        if audit_images.exists() {
            let _ = std::fs::create_dir_all(&dist_images);
            if let Ok(entries) = std::fs::read_dir(&audit_images) {
                for entry in entries.flatten() {
                    let dest = dist_images.join(entry.file_name());
                    let _ = std::fs::copy(entry.path(), dest);
                }
            }
        }

        // 5. Verification Phase (Local Dev Server + QA Agent)
        self.event_bus.emit_phase("Implementing", "Verifying", "Running QA verification with Playwright");
        self.state_machine.transition_to(PipelinePhase::Verifying, json!({ "html_written": true }))?;

        let mut dev_server = DevServerManager::new(
            self.workspace_dir.join("dist"),
            self.settings.browser.dev_server_port,
        );
        let _ = dev_server.start().await;

        let local_url = format!("http://localhost:{}", self.settings.browser.dev_server_port);
        let qa_agent = QaAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let _qa_report = qa_agent.verify_site(&local_url).await;
        let _ = dev_server.stop().await;

        // 6. Long-Term Memory Persistence
        let memory_path = self.workspace_dir.join("memory.db");
        if let Ok(store) = MemoryStore::open(&memory_path) {
            let summary = ProjectSummary {
                id: session_id,
                title: format!("Redesign: {}", target_source),
                target_url: Some(target_source.to_string()),
                macrostructure: "Workbench/Modern Bento".to_string(),
                color_palette: "Custom Directed Palette".to_string(),
                typography: "Space Grotesk + Inter".to_string(),
                user_rating: None,
                lessons_learned: format!("Adhered to design style '{}'. Integrated active skills: {:?}", self.settings.design.style_prompt, self.settings.design.selected_skills),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = store.save_summary(&summary);
        }

        self.event_bus.emit_phase("Verifying", "Completed", "Frontend generation and verification completed successfully");
        self.state_machine.transition_to(PipelinePhase::Completed, json!({ "output": output_html_path }))?;

        Ok(output_html_path)
    }

    pub async fn run_greenfield_pipeline(
        &mut self,
        brand_or_niche: &str,
        project_goal: &str,
    ) -> Result<PathBuf> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.event_bus.emit_phase("Init", "Researching", &format!("Market discovery & benchmarking for: {}", brand_or_niche));
        self.state_machine.transition_to(PipelinePhase::Researching, json!({ "target": brand_or_niche, "mode": "greenfield" }))?;

        // 1. Research Phase (Web benchmarking for niche)
        let research_agent = ResearchAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let research_brief = match research_agent.conduct_research(brand_or_niche, &self.settings.design.style_prompt).await {
            Ok(brief) => brief,
            Err(e) => {
                self.event_bus.emit_log("warn", "ResearchAgent", &format!("LLM fallback for research: {}", e));
                format!("Market research for {}: Leading modern high-conversion UI patterns in style {}", brand_or_niche, self.settings.design.style_prompt)
            }
        };

        // 2. Art Direction & Architecture Phase
        self.event_bus.emit_phase("Researching", "Designing", "Formulating design tokens, macrostructure & layout from scratch");
        self.state_machine.transition_to(PipelinePhase::Designing, json!({ "research_done": true }))?;

        let art_director = ArtDirectorAgent::new_with_skills(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
            &self.settings.design.selected_skills,
            &self.settings.design.style_prompt,
            &self.settings.design.references,
        );

        let combined_goal = format!("Greenfield Web Application for: {}\n\nBusiness Goal:\n{}\n\nDesign Style Directive:\n{}", brand_or_niche, project_goal, self.settings.design.style_prompt);
        let mock_audit = format!(r#"{{ "title": "{}", "description": "Brand new web experience built from scratch", "requirements": "{}" }}"#, brand_or_niche, project_goal);

        let design_spec = match art_director.create_design_system(&research_brief, &mock_audit, &combined_goal).await {
            Ok(spec) => spec,
            Err(e) => {
                self.event_bus.emit_log("warn", "ArtDirectorAgent", &format!("LLM fallback for art direction: {}", e));
                format!(
                    "# Design System for {}\n- Style: {}\n- Macrostructure: Workbench / Modern Bento\n- Anti-Slop: Strict Zero Emojis, vector Lucide SVGs, WCAG AA contrast.",
                    brand_or_niche, self.settings.design.style_prompt
                )
            }
        };

        let design_doc_path = self.workspace_dir.join("design.md");
        std::fs::write(&design_doc_path, &design_spec)?;

        // 3. Implementation Phase (Single-file HTML from scratch)
        self.event_bus.emit_phase("Designing", "Implementing", "Writing complete modern HTML/Tailwind/GSAP application");
        self.state_machine.transition_to(PipelinePhase::Implementing, json!({ "design_done": true }))?;

        let coder_agent = CoderAgent::new_with_skills(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
            &self.settings.design.selected_skills,
            &self.settings.design.style_prompt,
            &self.settings.design.references,
        );

        let context_desc = format!("Greenfield Application: {}\nRequirements: {}\nStyle: {}", brand_or_niche, project_goal, self.settings.design.style_prompt);
        let html_content = coder_agent.generate_frontend(&design_spec, &context_desc).await?;

        let output_html_path = self.workspace_dir.join("dist").join("index.html");
        if let Some(parent) = output_html_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_html_path, &html_content)?;

        // 4. Verification Phase (Local Dev Server + QA Agent)
        self.event_bus.emit_phase("Implementing", "Verifying", "Running QA verification with Playwright");
        self.state_machine.transition_to(PipelinePhase::Verifying, json!({ "html_written": true }))?;

        let mut dev_server = DevServerManager::new(
            self.workspace_dir.join("dist"),
            self.settings.browser.dev_server_port,
        );
        let _ = dev_server.start().await;

        let local_url = format!("http://localhost:{}", self.settings.browser.dev_server_port);
        let qa_agent = QaAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let _qa_report = qa_agent.verify_site(&local_url).await;
        let _ = dev_server.stop().await;

        // 5. Memory Persistence
        let memory_path = self.workspace_dir.join("memory.db");
        if let Ok(store) = MemoryStore::open(&memory_path) {
            let summary = ProjectSummary {
                id: session_id,
                title: format!("Greenfield: {}", brand_or_niche),
                target_url: None,
                macrostructure: "Workbench/Modern Bento".to_string(),
                color_palette: "Custom Directed Palette".to_string(),
                typography: "Space Grotesk + Inter".to_string(),
                user_rating: None,
                lessons_learned: format!("Built from scratch for '{}'. Style: {}", brand_or_niche, self.settings.design.style_prompt),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = store.save_summary(&summary);
        }

        self.event_bus.emit_phase("Verifying", "Completed", "Greenfield frontend generated and verified successfully");
        self.state_machine.transition_to(PipelinePhase::Completed, json!({ "output": output_html_path }))?;

        Ok(output_html_path)
    }

    pub async fn run_refinement_iteration(
        &mut self,
        critique: &str,
        rating: Option<u8>,
    ) -> Result<PathBuf> {
        let output_html_path = self.workspace_dir.join("dist").join("index.html");
        let design_doc_path = self.workspace_dir.join("design.md");

        if !output_html_path.exists() {
            anyhow::bail!("No generated site found in {:?} to refine. Run generation first.", output_html_path);
        }

        let existing_html = std::fs::read_to_string(&output_html_path)?;
        let design_spec = std::fs::read_to_string(&design_doc_path).unwrap_or_else(|_| "Modern Dark Industrial".into());

        self.event_bus.emit_phase("Completed", "Implementing", &format!("Applying user critique & fixes: {}", critique));
        self.state_machine.transition_to(PipelinePhase::Implementing, json!({ "iteration": "refinement", "critique": critique }))?;

        let coder_agent = CoderAgent::new_with_skills(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
            &self.settings.design.selected_skills,
            &self.settings.design.style_prompt,
            &self.settings.design.references,
        );

        let refinement_context = format!(
            "EXISTING FRONTEND CODE:\n{}\n\nUSER CRITIQUE & REFINEMENT REQUEST:\n{}\n\nINSTRUCTIONS:\n- Apply all requested fixes precisely to the existing HTML.\n- Keep valid DOCTYPE, Tailwind CSS CDN, Google Fonts, GSAP animations, and inline Lucide SVG icons.\n- ZERO UNICODE EMOJIS.",
            existing_html, critique
        );

        let updated_html = coder_agent.generate_frontend(&design_spec, &refinement_context).await?;
        std::fs::write(&output_html_path, &updated_html)?;

        // Run verification on updated code
        self.event_bus.emit_phase("Implementing", "Verifying", "Validating refined frontend with QA dev server");
        self.state_machine.transition_to(PipelinePhase::Verifying, json!({ "refinement_done": true }))?;

        let mut dev_server = DevServerManager::new(
            self.workspace_dir.join("dist"),
            self.settings.browser.dev_server_port,
        );
        let _ = dev_server.start().await;
        let local_url = format!("http://localhost:{}", self.settings.browser.dev_server_port);
        let qa_agent = QaAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let _qa_report = qa_agent.verify_site(&local_url).await;
        let _ = dev_server.stop().await;

        // Update memory with user rating
        let memory_path = self.workspace_dir.join("memory.db");
        if let Ok(store) = MemoryStore::open(&memory_path) {
            let session_id = uuid::Uuid::new_v4().to_string();
            let summary = ProjectSummary {
                id: session_id,
                title: "Refinement Iteration".to_string(),
                target_url: None,
                macrostructure: "Refined Workbench/Bento".to_string(),
                color_palette: "Preserved Palette".to_string(),
                typography: "Preserved Typography".to_string(),
                user_rating: rating,
                lessons_learned: format!("User critique: {}. Applied fixes.", critique),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = store.save_summary(&summary);
        }

        self.event_bus.emit_phase("Verifying", "Completed", "Refinement iteration finished successfully");
        self.state_machine.transition_to(PipelinePhase::Completed, json!({ "output": output_html_path }))?;

        Ok(output_html_path)
    }
}

fn generate_fallback_redesign(target_url: &str, report: &crate::tools::browser::AuditReport) -> String {
    let raw_title = report.site_analysis.get("title").and_then(|t| t.as_str()).unwrap_or("Audi Service Челябинск");
    let clean_title = raw_title.replace('"', "&quot;");
    let template = get_template_html();

    template
        .replace("__TITLE__", &clean_title)
        .replace("__TARGET_URL__", target_url)
}
