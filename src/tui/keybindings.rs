use crate::tui::state::{ActiveModal, FocusedPane, TuiState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct KeybindingHandler;

impl KeybindingHandler {
    pub fn handle_key(state: &mut TuiState, key: KeyEvent) {
        // Special modal: Config Editor
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
                        // Space cycles selectable options (Search Engine, Reasoning Effort, Headless)
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
