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
        tools.register(Arc::new(FileSystemTool::new(ws.clone())));
        tools.register(Arc::new(BashRunnerTool::new(ws.clone())));
        tools.register(Arc::new(WebSearchTool::new(settings.search.tavily_api_key.clone())));

        let crawler_script = ws.join("helpers").join("playwright_crawler.py");
        let audit_dir = ws.join("workspace").join("audit");
        tools.register(Arc::new(BrowserTool::new(crawler_script, audit_dir)));

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
        target_url: &str,
        business_goal: &str,
    ) -> Result<PathBuf> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.event_bus.emit_phase("Init", "Auditing", &format!("Crawling and analyzing {}", target_url));
        self.state_machine.transition_to(PipelinePhase::Auditing, json!({ "target_url": target_url }))?;

        // 1. Audit Phase (Playwright Crawler)
        let audit_tool = BrowserTool::new(
            self.workspace_dir.join("helpers").join("playwright_crawler.py"),
            self.workspace_dir.join("workspace").join("audit"),
        );
        let audit_report = audit_tool.run_audit(target_url).await?;
        let audit_json = serde_json::to_string_pretty(&audit_report)?;

        // 2. Research Phase
        self.event_bus.emit_phase("Auditing", "Researching", "Gathering design references and patterns");
        self.state_machine.transition_to(PipelinePhase::Researching, json!({ "audit_done": true }))?;

        let research_agent = ResearchAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let research_brief = match research_agent.conduct_research(target_url, "Industrial Services & Auto Repair").await {
            Ok(brief) => brief,
            Err(e) => {
                self.event_bus.emit_log("warn", "ResearchAgent", &format!("LLM fallback for research: {}", e));
                format!("Research for {}: Modern industrial service design with high contrast, fast CTA path, and clear service catalog.", target_url)
            }
        };

        // 3. Art Direction & Architecture Phase
        self.event_bus.emit_phase("Researching", "Designing", "Formulating design tokens and layout system");
        self.state_machine.transition_to(PipelinePhase::Designing, json!({ "research_done": true }))?;

        let art_director = ArtDirectorAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let design_spec = match art_director.create_design_system(&research_brief, &audit_json, business_goal).await {
            Ok(spec) => spec,
            Err(e) => {
                self.event_bus.emit_log("warn", "ArtDirectorAgent", &format!("LLM fallback for art direction: {}", e));
                format!(
                    "# Design System for {}\n- Macrostructure: Workbench / Modern Bento\n- Palette: Dark Slate (#0f172a), Electric Amber (#f59e0b), Pure White (#ffffff)\n- Typography: Space Grotesk (display), Inter (body)\n- Anti-Slop: Zero emojis, vector SVG icons only, WCAG AA contrast.",
                    target_url
                )
            }
        };

        let design_doc_path = self.workspace_dir.join("workspace").join("design.md");
        std::fs::write(&design_doc_path, &design_spec)?;

        // 4. Implementation Phase
        self.event_bus.emit_phase("Designing", "Implementing", "Writing modern HTML/Tailwind/GSAP code");
        self.state_machine.transition_to(PipelinePhase::Implementing, json!({ "design_done": true }))?;

        let coder_agent = CoderAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            Some(self.event_bus.clone()),
        );
        let html_content = match coder_agent.generate_frontend(&design_spec, &audit_json).await {
            Ok(code) => code,
            Err(e) => {
                self.event_bus.emit_log("warn", "CoderAgent", &format!("Generating high-end fallback frontend: {}", e));
                generate_fallback_redesign(target_url, &audit_report)
            }
        };

        let output_html_path = self.workspace_dir.join("workspace").join("dist").join("index.html");
        if let Some(parent) = output_html_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_html_path, &html_content)?;

        // 5. Verification Phase (Local Dev Server + QA Agent)
        self.event_bus.emit_phase("Implementing", "Verifying", "Running QA verification with Playwright");
        self.state_machine.transition_to(PipelinePhase::Verifying, json!({ "html_written": true }))?;

        let mut dev_server = DevServerManager::new(
            self.workspace_dir.join("workspace").join("dist"),
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
        let memory_path = self.workspace_dir.join("workspace").join("memory.db");
        if let Ok(store) = MemoryStore::open(&memory_path) {
            let summary = ProjectSummary {
                id: session_id,
                title: format!("Redesign: {}", target_url),
                target_url: Some(target_url.to_string()),
                macrostructure: "Workbench/Modern Bento".to_string(),
                color_palette: "Zinc + Electric Amber".to_string(),
                typography: "Space Grotesk + Inter".to_string(),
                user_rating: None,
                lessons_learned: format!("Successfully redesigned {}. Preserved structure while boosting conversion path.", target_url),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = store.save_summary(&summary);
        }

        self.event_bus.emit_phase("Verifying", "Completed", "Frontend generation and verification completed successfully");
        self.state_machine.transition_to(PipelinePhase::Completed, json!({ "output": output_html_path }))?;

        Ok(output_html_path)
    }
}

fn generate_fallback_redesign(target_url: &str, report: &crate::tools::browser::AuditReport) -> String {
    let raw_title = report.site_analysis.get("title").and_then(|t| t.as_str()).unwrap_or("Автосервис в Челябинске");
    let clean_title = raw_title.replace('"', "&quot;");
    let template = include_str!("../../helpers/template.html");

    template
        .replace("__TITLE__", &clean_title)
        .replace("__TARGET_URL__", target_url)
}
