use crate::config::settings::Settings;
use crate::core::events::Event;
use crate::llm::reasoning::{cycle_next_effort, get_available_efforts_for_model};
use crate::skills::registry::{SkillItem, SkillRegistry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedPane {
    DagTree,
    StreamBuffer,
    LogBuffer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    Help,
    Diff,
    Logs,
    Assets,
    Memory,
    Config,
    NewRun,
}

#[derive(Debug, Clone)]
pub struct DagStep {
    pub id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct ConfigField {
    pub key: String,
    pub label: String,
    pub value: String,
    pub is_secret: bool,
    pub description: String,
}

pub struct TuiState {
    pub focused_pane: FocusedPane,
    pub active_modal: ActiveModal,
    pub stream_buffer: String,
    pub logs: Vec<String>,
    pub dag_steps: Vec<DagStep>,
    pub current_phase: String,
    pub model_name: String,
    pub reasoning_effort: String,
    pub dev_server_status: String,
    pub scroll_offset: usize,
    pub should_quit: bool,

    // Config editor state
    pub config_fields: Vec<ConfigField>,
    pub config_selected_index: usize,
    pub is_editing_field: bool,
    pub config_edit_buffer: String,
    pub config_status_message: Option<String>,

    // New Run / Design Studio modal state
    pub run_target_source: String,
    pub run_workspace_dir: String,
    pub run_goal_prompt: String,
    pub run_design_style: String,
    pub run_references: String,
    pub run_skills: Vec<SkillItem>,
    pub run_skills_cursor: usize,
    pub run_input_focus: usize, // 0: Source, 1: Workspace, 2: Goal, 3: Style, 4: References, 5: Skills, 6: Launch
    pub should_trigger_pipeline: Option<(String, String, Settings)>,
}

impl TuiState {
    pub fn new(model_name: &str, reasoning_effort: &str) -> Self {
        let settings = Settings::load().unwrap_or_default();
        let available_efforts = get_available_efforts_for_model(&settings.llm.model).join(", ");
        let config_fields = vec![
            ConfigField {
                key: "LLM_BASE_URL".into(),
                label: "LLM Base URL".into(),
                value: settings.llm.base_url.clone(),
                is_secret: false,
                description: "OpenAI-compatible base API endpoint".into(),
            },
            ConfigField {
                key: "LLM_API_KEY".into(),
                label: "LLM API Key".into(),
                value: settings.llm.api_key.clone(),
                is_secret: true,
                description: "Bearer authentication key for provider".into(),
            },
            ConfigField {
                key: "LLM_MODEL".into(),
                label: "LLM Model".into(),
                value: settings.llm.model.clone(),
                is_secret: false,
                description: "Target model ID (e.g. gpt-5.6-sol, claude-3-5-sonnet, o3-mini)".into(),
            },
            ConfigField {
                key: "LLM_REASONING_EFFORT".into(),
                label: "Reasoning Effort".into(),
                value: settings.llm.reasoning_effort.clone(),
                is_secret: false,
                description: format!("Available: {} [Press Space to cycle]", available_efforts),
            },
            ConfigField {
                key: "SEARCH_PROVIDER".into(),
                label: "Search Engine".into(),
                value: settings.search.provider.clone(),
                is_secret: false,
                description: "Provider: 'duckduckgo' (free/keyless) or 'tavily' [Press Space to toggle]".into(),
            },
            ConfigField {
                key: "TAVILY_API_KEY".into(),
                label: "Tavily API Key".into(),
                value: settings.search.tavily_api_key.clone().unwrap_or_default(),
                is_secret: true,
                description: "Tavily Search API key (optional if using DuckDuckGo)".into(),
            },
            ConfigField {
                key: "WORKSPACE_DIR".into(),
                label: "Workspace Directory".into(),
                value: settings.workspace_dir.to_string_lossy().to_string(),
                is_secret: false,
                description: "Default output directory for generated sites and artifacts".into(),
            },
            ConfigField {
                key: "DEV_SERVER_PORT".into(),
                label: "Dev Server Port".into(),
                value: settings.browser.dev_server_port.to_string(),
                is_secret: false,
                description: "Local port for QA dev server (e.g. 3000)".into(),
            },
            ConfigField {
                key: "BROWSER_HEADLESS".into(),
                label: "Browser Headless".into(),
                value: if settings.browser.headless { "true".into() } else { "false".into() },
                is_secret: false,
                description: "Run Playwright browser in background (true/false) [Press Space to toggle]".into(),
            },
        ];

        Self {
            focused_pane: FocusedPane::StreamBuffer,
            active_modal: ActiveModal::None,
            stream_buffer: String::new(),
            logs: Vec::new(),
            dag_steps: vec![
                DagStep { id: "1".into(), name: "1. Audit (Playwright)".into(), status: "PENDING".into(), detail: "Inspect DOM, CSS & assets".into() },
                DagStep { id: "2".into(), name: "2. Research (Web)".into(), status: "PENDING".into(), detail: "Discover industry patterns".into() },
                DagStep { id: "3".into(), name: "3. Art Direction".into(), status: "PENDING".into(), detail: "Build tokens & macrostructure".into() },
                DagStep { id: "4".into(), name: "4. Implementation".into(), status: "PENDING".into(), detail: "Write modern HTML/Tailwind/GSAP".into() },
                DagStep { id: "5".into(), name: "5. QA & Verification".into(), status: "PENDING".into(), detail: "Playwright local test".into() },
            ],
            current_phase: "Idle".to_string(),
            model_name: model_name.to_string(),
            reasoning_effort: reasoning_effort.to_string(),
            dev_server_status: "Stopped".to_string(),
            scroll_offset: 0,
            should_quit: false,
            config_fields,
            config_selected_index: 0,
            is_editing_field: false,
            config_edit_buffer: String::new(),
            config_status_message: None,

            // New Run State
            run_target_source: "https://as-chelyabinsk.ru/".into(),
            run_workspace_dir: "workspace".into(),
            run_goal_prompt: "Повысить конверсию, сделать современный сайт с Bento-сеткой и плавными анимациями".into(),
            run_design_style: "Modern Dark Industrial: Dark Slate (#09090b), Electric Amber (#f59e0b) accent, Space Grotesk headings, Inter body, GSAP ScrollTrigger, Lucide vector icons".into(),
            run_references: "https://linear.app, https://audi.com".into(),
            run_skills: SkillRegistry::default_skills(),
            run_skills_cursor: 0,
            run_input_focus: 0,
            should_trigger_pipeline: None,
        }
    }

    pub fn toggle_selected_skill(&mut self) {
        let idx = self.run_skills_cursor;
        if idx < self.run_skills.len() {
            self.run_skills[idx].enabled = !self.run_skills[idx].enabled;
        }
    }

    pub fn toggle_or_cycle_selected_field(&mut self) {
        let idx = self.config_selected_index;
        if idx >= self.config_fields.len() {
            return;
        }

        let key = self.config_fields[idx].key.clone();
        let current_val = self.config_fields[idx].value.clone();

        match key.as_str() {
            "SEARCH_PROVIDER" => {
                let next = if current_val.to_lowercase() == "tavily" {
                    "duckduckgo"
                } else {
                    "tavily"
                };
                self.config_fields[idx].value = next.to_string();
                self.config_status_message = Some(format!("Search engine switched to '{}'. Press 's' to save.", next));
            }
            "BROWSER_HEADLESS" => {
                let next = if current_val.to_lowercase() == "true" {
                    "false"
                } else {
                    "true"
                };
                self.config_fields[idx].value = next.to_string();
                self.config_status_message = Some(format!("Browser headless mode set to '{}'. Press 's' to save.", next));
            }
            "LLM_REASONING_EFFORT" => {
                let model_field_val = self.config_fields.iter().find(|f| f.key == "LLM_MODEL").map(|f| f.value.as_str()).unwrap_or(&self.model_name);
                let next_effort = cycle_next_effort(&current_val, model_field_val);
                self.config_fields[idx].value = next_effort.clone();
                self.reasoning_effort = next_effort.clone();
                self.config_status_message = Some(format!("Reasoning effort set to '{}'. Press 's' to save.", next_effort));
            }
            _ => {}
        }
    }

    pub fn refresh_model_efforts(&mut self) {
        let model = self.config_fields.iter().find(|f| f.key == "LLM_MODEL").map(|f| f.value.clone()).unwrap_or_else(|| self.model_name.clone());
        let available = get_available_efforts_for_model(&model).join(", ");
        if let Some(field) = self.config_fields.iter_mut().find(|f| f.key == "LLM_REASONING_EFFORT") {
            field.description = format!("Available for {}: {} [Press Space to cycle]", model, available);
        }
    }

    pub fn save_config_fields(&mut self) {
        let mut settings = Settings::load().unwrap_or_default();
        let mut env_content = String::new();

        for f in &self.config_fields {
            env_content.push_str(&format!("{}={}\n", f.key, f.value));
            match f.key.as_str() {
                "LLM_BASE_URL" => settings.llm.base_url = f.value.clone(),
                "LLM_API_KEY" => settings.llm.api_key = f.value.clone(),
                "LLM_MODEL" => {
                    settings.llm.model = f.value.clone();
                    self.model_name = f.value.clone();
                }
                "LLM_REASONING_EFFORT" => {
                    settings.llm.reasoning_effort = f.value.clone();
                    self.reasoning_effort = f.value.clone();
                }
                "SEARCH_PROVIDER" => {
                    settings.search.provider = f.value.to_lowercase();
                }
                "TAVILY_API_KEY" => {
                    settings.search.tavily_api_key = if f.value.is_empty() { None } else { Some(f.value.clone()) };
                }
                "WORKSPACE_DIR" => {
                    settings.workspace_dir = std::path::PathBuf::from(&f.value);
                }
                "DEV_SERVER_PORT" => {
                    if let Ok(p) = f.value.parse::<u16>() {
                        settings.browser.dev_server_port = p;
                    }
                }
                "BROWSER_HEADLESS" => {
                    settings.browser.headless = f.value.to_lowercase() == "true" || f.value == "1";
                }
                _ => {}
            }
        }

        let _ = settings.save_global_config();
        let env_path = settings.workspace_dir.join(".env");
        let _ = std::fs::write(&env_path, env_content);

        self.refresh_model_efforts();
        self.config_status_message = Some("Settings saved to config.yaml and .env successfully!".to_string());
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::TokenStream { chunk, .. } => {
                self.stream_buffer.push_str(chunk);
            }
            Event::PhaseChange { from: _, to, description } => {
                self.current_phase = to.clone();
                self.logs.push(format!("[Phase] {} -> {}: {}", self.current_phase, to, description));
                self.update_dag_step_for_phase(to);
            }
            Event::LogMessage { level, source, message } => {
                self.logs.push(format!("[{}] {}: {}", level.to_uppercase(), source, message));
            }
            Event::DagStepUpdate { step_id, name: _, status, detail } => {
                for s in &mut self.dag_steps {
                    if s.id == *step_id {
                        s.status = status.clone();
                        s.detail = detail.clone();
                    }
                }
            }
            Event::Error { source, message } => {
                self.logs.push(format!("[ERROR] {}: {}", source, message));
            }
            _ => {}
        }
    }

    fn update_dag_step_for_phase(&mut self, phase: &str) {
        match phase {
            "Auditing" => {
                if let Some(s) = self.dag_steps.get_mut(0) { s.status = "RUNNING".into(); }
            }
            "Researching" => {
                if let Some(s) = self.dag_steps.get_mut(0) { s.status = "DONE".into(); }
                if let Some(s) = self.dag_steps.get_mut(1) { s.status = "RUNNING".into(); }
            }
            "Designing" => {
                if let Some(s) = self.dag_steps.get_mut(1) { s.status = "DONE".into(); }
                if let Some(s) = self.dag_steps.get_mut(2) { s.status = "RUNNING".into(); }
            }
            "Implementing" => {
                if let Some(s) = self.dag_steps.get_mut(2) { s.status = "DONE".into(); }
                if let Some(s) = self.dag_steps.get_mut(3) { s.status = "RUNNING".into(); }
            }
            "Verifying" => {
                if let Some(s) = self.dag_steps.get_mut(3) { s.status = "DONE".into(); }
                if let Some(s) = self.dag_steps.get_mut(4) { s.status = "RUNNING".into(); }
            }
            "Completed" => {
                if let Some(s) = self.dag_steps.get_mut(4) { s.status = "DONE".into(); }
            }
            _ => {}
        }
    }
}
