use crate::config::settings::Settings;
use crate::tui::state::{ActiveModal, FocusedPane, TuiState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct KeybindingHandler;

impl KeybindingHandler {
    pub fn handle_key(state: &mut TuiState, key: KeyEvent) {
        // Modal: Design Studio / New Run
        if state.active_modal == ActiveModal::NewRun {
            match key.code {
                KeyCode::Esc => {
                    state.active_modal = ActiveModal::None;
                }
                KeyCode::Tab => {
                    state.run_input_focus = (state.run_input_focus + 1) % 6;
                }
                KeyCode::BackTab => {
                    if state.run_input_focus == 0 {
                        state.run_input_focus = 5;
                    } else {
                        state.run_input_focus -= 1;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') if state.run_input_focus == 5 => {
                    if state.run_skills_cursor == 0 {
                        state.run_skills_cursor = state.run_skills.len().saturating_sub(1);
                    } else {
                        state.run_skills_cursor -= 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') if state.run_input_focus == 5 => {
                    if !state.run_skills.is_empty() {
                        state.run_skills_cursor = (state.run_skills_cursor + 1) % state.run_skills.len();
                    }
                }
                KeyCode::Char(' ') if state.run_input_focus == 5 => {
                    state.toggle_selected_skill();
                }
                KeyCode::Enter => {
                    let source = state.run_target_source.trim().to_string();
                    let goal = state.run_goal_prompt.trim().to_string();
                    if !source.is_empty() {
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

                        state.should_trigger_pipeline = Some((source, goal, settings));
                        state.active_modal = ActiveModal::None;
                    }
                }
                KeyCode::Backspace => {
                    match state.run_input_focus {
                        0 => { state.run_target_source.pop(); }
                        1 => { state.run_workspace_dir.pop(); }
                        2 => { state.run_goal_prompt.pop(); }
                        3 => { state.run_design_style.pop(); }
                        4 => { state.run_references.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Char(ch) => {
                    match state.run_input_focus {
                        0 => { state.run_target_source.push(ch); }
                        1 => { state.run_workspace_dir.push(ch); }
                        2 => { state.run_goal_prompt.push(ch); }
                        3 => { state.run_design_style.push(ch); }
                        4 => { state.run_references.push(ch); }
                        _ => {}
                    }
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

        // Other modal navigation
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
}
