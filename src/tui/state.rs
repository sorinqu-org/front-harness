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
}

#[derive(Debug, Clone)]
pub struct DagStep {
    pub id: String,
    pub name: String,
    pub status: String,
    pub detail: String,
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
}

impl TuiState {
    pub fn new(model_name: &str, reasoning_effort: &str) -> Self {
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
        }
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
