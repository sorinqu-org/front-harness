use crate::config::settings::Settings;
use crate::memory::store::{MemoryStore, ProjectSummary};
use crate::tui::state::{ActiveModal, FocusedPane, RunMode, TuiState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct KeybindingHandler;

impl KeybindingHandler {
    pub fn handle_key(state: &mut TuiState, key: KeyEvent) {
        // Modal: Design Studio / New Run
        if state.active_modal == ActiveModal::NewRun {
            // Check for immediate launch shortcut (Ctrl+S or Ctrl+Enter)
            if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('s') || key.code == KeyCode::Enter) {
                Self::trigger_pipeline_launch(state);
                return;
            }

            match key.code {
                KeyCode::Esc => {
                    state.active_modal = ActiveModal::None;
                }
                KeyCode::Char('m') => {
                    state.run_mode = match state.run_mode {
                        RunMode::Redesign => RunMode::Greenfield,
                        RunMode::Greenfield => RunMode::Redesign,
                    };
                }
                KeyCode::Tab | KeyCode::Down => {
                    state.run_input_focus = (state.run_input_focus + 1) % 8;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    if state.run_input_focus == 0 {
                        state.run_input_focus = 7;
                    } else {
                        state.run_input_focus -= 1;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') if state.run_input_focus == 0 => {
                    state.run_mode = RunMode::Redesign;
                }
                KeyCode::Right | KeyCode::Char('l') if state.run_input_focus == 0 => {
                    state.run_mode = RunMode::Greenfield;
                }
                KeyCode::Left | KeyCode::Char('h') if state.run_input_focus == 6 => {
                    if state.run_skills_cursor == 0 {
                        state.run_skills_cursor = state.run_skills.len().saturating_sub(1);
                    } else {
                        state.run_skills_cursor -= 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') if state.run_input_focus == 6 => {
                    if !state.run_skills.is_empty() {
                        state.run_skills_cursor = (state.run_skills_cursor + 1) % state.run_skills.len();
                    }
                }
                KeyCode::Char(' ') if state.run_input_focus == 6 => {
                    state.toggle_selected_skill();
                }
                KeyCode::Char(' ') if state.run_input_focus == 7 => {
                    Self::trigger_pipeline_launch(state);
                }
                KeyCode::Enter => {
                    if state.run_input_focus == 7 {
                        Self::trigger_pipeline_launch(state);
                    } else if state.run_input_focus == 6 {
                        state.toggle_selected_skill();
                    } else {
                        state.run_input_focus = (state.run_input_focus + 1) % 8;
                    }
                }
                KeyCode::Backspace => {
                    match state.run_input_focus {
                        1 => { state.run_target_source.pop(); }
                        2 => { state.run_workspace_dir.pop(); }
                        3 => { state.run_goal_prompt.pop(); }
                        4 => { state.run_design_style.pop(); }
                        5 => { state.run_references.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Char(ch) => {
                    match state.run_input_focus {
                        1 => { state.run_target_source.push(ch); }
                        2 => { state.run_workspace_dir.push(ch); }
                        3 => { state.run_goal_prompt.push(ch); }
                        4 => { state.run_design_style.push(ch); }
                        5 => { state.run_references.push(ch); }
                        _ => {}
                    }
                }
                _ => {}
            }
            return;
        }

        // Modal: Project Review & Refinement
        if state.active_modal == ActiveModal::Review {
            if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('s') || key.code == KeyCode::Enter) {
                Self::trigger_refinement_launch(state);
                return;
            }

            match key.code {
                KeyCode::Esc => {
                    state.active_modal = ActiveModal::None;
                }
                KeyCode::Char(c @ '1'..='5') if state.review_focus != 1 => {
                    if let Some(digit) = c.to_digit(10) {
                        state.review_rating = digit as u8;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') if state.review_focus == 0 => {
                    if state.review_rating > 1 {
                        state.review_rating -= 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') if state.review_focus == 0 => {
                    if state.review_rating < 5 {
                        state.review_rating += 1;
                    }
                }
                KeyCode::Tab | KeyCode::Down => {
                    state.review_focus = (state.review_focus + 1) % 4;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    if state.review_focus == 0 {
                        state.review_focus = 3;
                    } else {
                        state.review_focus -= 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') if state.review_focus == 2 => {
                    Self::trigger_refinement_launch(state);
                }
                KeyCode::Enter | KeyCode::Char(' ') if state.review_focus == 3 => {
                    Self::save_review_feedback_to_memory(state);
                    state.active_modal = ActiveModal::None;
                }
                KeyCode::Enter if state.review_focus == 1 => {
                    state.review_focus = 2;
                }
                KeyCode::Backspace if state.review_focus == 1 => {
                    state.review_critique.pop();
                }
                KeyCode::Char(ch) if state.review_focus == 1 => {
                    state.review_critique.push(ch);
                }
                _ => {}
            }
            return;
        }

        // Modal: Config Editor
        if state.active_modal == ActiveModal::Config {
            if state.is_editing_field {
                match key.code {
                    KeyCode::Enter => {
                        let idx = state.config_selected_index;
                        if idx < state.config_fields.len() {
                            state.config_fields[idx].value = state.config_edit_buffer.clone();
                            if state.config_fields[idx].key == "LLM_MODEL" {
                                state.refresh_model_efforts();
                            }
                        }
                        state.is_editing_field = false;
                        state.config_status_message = Some("Value updated in memory. Press 's' to save to disk.".into());
                    }
                    KeyCode::Esc => {
                        state.is_editing_field = false;
                        state.config_status_message = None;
                    }
                    KeyCode::Backspace => {
                        state.config_edit_buffer.pop();
                    }
                    KeyCode::Char(ch) => {
                        state.config_edit_buffer.push(ch);
                    }
                    _ => {}
                }
                return;
            } else {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        state.active_modal = ActiveModal::None;
                        state.config_status_message = None;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if !state.config_fields.is_empty() {
                            state.config_selected_index = (state.config_selected_index + 1) % state.config_fields.len();
                        }
                        state.config_status_message = None;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if !state.config_fields.is_empty() {
                            if state.config_selected_index == 0 {
                                state.config_selected_index = state.config_fields.len() - 1;
                            } else {
                                state.config_selected_index -= 1;
                            }
                        }
                        state.config_status_message = None;
                    }
                    KeyCode::Char(' ') => {
                        state.toggle_or_cycle_selected_field();
                    }
                    KeyCode::Enter | KeyCode::Char('i') => {
                        let idx = state.config_selected_index;
                        if idx < state.config_fields.len() {
                            let key = state.config_fields[idx].key.as_str();
                            if key == "SEARCH_PROVIDER" || key == "BROWSER_HEADLESS" || key == "LLM_REASONING_EFFORT" {
                                state.toggle_or_cycle_selected_field();
                            } else {
                                state.config_edit_buffer = state.config_fields[idx].value.clone();
                                state.is_editing_field = true;
                                state.config_status_message = None;
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        state.save_config_fields();
                    }
                    _ => {}
                }
                return;
            }
        }

        // Other modals
        if state.active_modal != ActiveModal::None {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    state.active_modal = ActiveModal::None;
                }
                _ => {}
            }
            return;
        }

        // Main screen keybindings
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Char('q')) => {
                state.should_quit = true;
            }
            (KeyModifiers::NONE, KeyCode::Char('?')) => {
                state.active_modal = ActiveModal::Help;
            }
            (KeyModifiers::NONE, KeyCode::Char('r')) | (KeyModifiers::NONE, KeyCode::Char('n')) => {
                state.active_modal = ActiveModal::NewRun;
            }
            (KeyModifiers::NONE, KeyCode::Char('e')) | (KeyModifiers::NONE, KeyCode::Char('f')) => {
                state.active_modal = ActiveModal::Review;
            }
            (KeyModifiers::NONE, KeyCode::Char('c')) => {
                state.active_modal = ActiveModal::Config;
            }
            (KeyModifiers::NONE, KeyCode::Char('d')) => {
                state.active_modal = ActiveModal::Diff;
            }
            (KeyModifiers::NONE, KeyCode::Char('l')) => {
                state.active_modal = ActiveModal::Logs;
            }
            (KeyModifiers::NONE, KeyCode::Char('a')) => {
                state.active_modal = ActiveModal::Assets;
            }
            (KeyModifiers::NONE, KeyCode::Char('m')) => {
                state.active_modal = ActiveModal::Memory;
            }
            (KeyModifiers::NONE, KeyCode::Char('o')) => {
                Self::open_in_browser(state);
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                state.focused_pane = match state.focused_pane {
                    FocusedPane::DagTree => FocusedPane::StreamBuffer,
                    FocusedPane::StreamBuffer => FocusedPane::LogBuffer,
                    FocusedPane::LogBuffer => FocusedPane::DagTree,
                };
            }
            (KeyModifiers::NONE, KeyCode::Char('j')) | (KeyModifiers::NONE, KeyCode::Down) => {
                state.scroll_offset = state.scroll_offset.saturating_add(1);
            }
            (KeyModifiers::NONE, KeyCode::Char('k')) | (KeyModifiers::NONE, KeyCode::Up) => {
                state.scroll_offset = state.scroll_offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn trigger_pipeline_launch(state: &mut TuiState) {
        let source_or_niche = state.run_target_source.trim().to_string();
        let goal = state.run_goal_prompt.trim().to_string();
        if !source_or_niche.is_empty() {
            let mut settings = Settings::load().unwrap_or_default();
            settings.workspace_dir = std::path::PathBuf::from(&state.run_workspace_dir);
            settings.design.style_prompt = state.run_design_style.clone();
            settings.design.references = state
                .run_references
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            settings.design.selected_skills = state
                .run_skills
                .iter()
                .filter(|s| s.enabled)
                .map(|s| s.id.to_string())
                .collect();

            match state.run_mode {
                RunMode::Redesign => {
                    state.reset_for_new_run(&source_or_niche);
                    state.should_trigger_pipeline = Some((source_or_niche, goal, settings));
                }
                RunMode::Greenfield => {
                    state.reset_for_greenfield_run(&source_or_niche);
                    state.should_trigger_greenfield = Some((source_or_niche, goal, settings));
                }
            }
            state.active_modal = ActiveModal::None;
        }
    }

    fn trigger_refinement_launch(state: &mut TuiState) {
        let critique = state.review_critique.trim().to_string();
        if !critique.is_empty() {
            let settings = Settings::load().unwrap_or_default();
            let rating = Some(state.review_rating);
            state.reset_for_refinement_run(&critique);
            state.should_trigger_refinement = Some((critique, rating, settings));
            state.active_modal = ActiveModal::None;
        }
    }

    fn save_review_feedback_to_memory(state: &TuiState) {
        let memory_path = std::path::PathBuf::from(&state.run_workspace_dir).join("memory.db");
        if let Ok(store) = MemoryStore::open(&memory_path) {
            let session_id = uuid::Uuid::new_v4().to_string();
            let summary = ProjectSummary {
                id: session_id,
                title: "User Evaluation & Critique".to_string(),
                target_url: Some(state.run_target_source.clone()),
                macrostructure: "User Rated".to_string(),
                color_palette: state.run_design_style.clone(),
                typography: "Active Typography".to_string(),
                user_rating: Some(state.review_rating),
                lessons_learned: format!("Rating: {}/5. Feedback: {}", state.review_rating, state.review_critique),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = store.save_summary(&summary);
        }
    }

    fn open_in_browser(state: &mut TuiState) {
        let dist_path = std::path::PathBuf::from(&state.run_workspace_dir).join("dist").join("index.html");
        if dist_path.exists() {
            let url = if state.is_dev_server_running {
                format!("http://localhost:{}", state.dev_server_port)
            } else {
                format!("file://{}", dist_path.display())
            };
            let _ = std::process::Command::new("xdg-open")
                .arg(&url)
                .spawn();
            state.logs.push(format!("[Browser] Opened {} in default web browser", url));
        } else {
            state.logs.push("[Browser] Output file dist/index.html does not exist yet. Run generation first.".into());
        }
    }
}
