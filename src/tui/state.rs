use crate::config::settings::Settings;
use crate::core::events::Event;

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
}

impl TuiState {
    pub fn new(model_name: &str, reasoning_effort: &str) -> Self {
        let settings = Settings::load().unwrap_or_default();
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
                description: "Target model ID (e.g. gpt-5.6-sol, claude-3-5-sonnet)".into(),
            },
            ConfigField {
                key: "LLM_REASONING_EFFORT".into(),
                label: "Reasoning Effort".into(),
                value: settings.llm.reasoning_effort.clone(),
                is_secret: false,
                description: "Effort level: low, medium, high, custom".into(),
            },
            ConfigField {
                key: "TAVILY_API_KEY".into(),
                label: "Tavily API Key".into(),
                value: settings.search.tavily_api_key.clone().unwrap_or_default(),
                is_secret: true,
                description: "Tavily Search API key for web research".into(),
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
                description: "Run Playwright browser in background (true/false)".into(),
            },
        ];

        Self {
            focused_pane: FocusedPane::StreamBuffer,
            active_modal: ActiveModal::None,
            stream_buffer: String::new(),
            logs: Vec::new(),
            dag_steps: vec![
                DagStep { id: "1".into(), name: "1. Audit (Playwright)".into(), status: "PENDING".into(), detail: "Inspect DOM & capture screenshots".into() },
                DagStep { id: "2".into(), name: "2. Research (Tavily)".into(), status: "PENDING".into(), detail: "Discover industry patterns".into() },
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
                "TAVILY_API_KEY" => {
                    settings.search.tavily_api_key = if f.value.is_empty() { None } else { Some(f.value.clone()) };
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

        // Save global config yaml
        let _ = settings.save_global_config();

        // Also save to .env in current workspace
        let env_path = settings.workspace_dir.join(".env");
        let _ = std::fs::write(&env_path, env_content);

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
