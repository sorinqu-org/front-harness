use crate::tui::state::{ActiveModal, FocusedPane, TuiState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct KeybindingHandler;

impl KeybindingHandler {
    pub fn handle_key(state: &mut TuiState, key: KeyEvent) {
        // Modal navigation takes priority
        if state.active_modal != ActiveModal::None {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    state.active_modal = ActiveModal::None;
                }
                _ => {}
            }
            return;
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Char('q')) => {
                state.should_quit = true;
            }
            (KeyModifiers::NONE, KeyCode::Char('?')) => {
                state.active_modal = ActiveModal::Help;
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
