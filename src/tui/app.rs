use crate::agents::PipelineOrchestrator;
use crate::config::settings::Settings;
use crate::core::event_bus::EventBus;
use crate::tui::keybindings::KeybindingHandler;
use crate::tui::state::{ActiveModal, TuiState};
use crate::tui::widgets::{
    render_asset_modal, render_config_modal, render_dag_tree, render_diff_modal, render_help_modal,
    render_log_modal, render_memory_modal, render_new_run_modal, render_statusline, render_stream_view,
};
use anyhow::Result;
use crossterm::{
    event::{EventStream, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::stdout;
use std::time::Duration;

pub struct TuiApp {
    state: TuiState,
    event_bus: EventBus,
}

impl TuiApp {
    pub fn new(model_name: &str, reasoning_effort: &str, event_bus: EventBus) -> Self {
        Self {
            state: TuiState::new(model_name, reasoning_effort),
            event_bus,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut event_reader = EventStream::new();
        let mut bus_receiver = self.event_bus.subscribe();

        loop {
            terminal.draw(|f| {
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(5), Constraint::Length(1)])
                    .split(size);

                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
                    .split(chunks[0]);

                render_dag_tree(f, main_chunks[0], &self.state);
                render_stream_view(f, main_chunks[1], &self.state);
                render_statusline(f, chunks[1], &self.state);

                // Render Modals
                match self.state.active_modal {
                    ActiveModal::Help => render_help_modal(f),
                    ActiveModal::NewRun => render_new_run_modal(f, &self.state),
                    ActiveModal::Config => render_config_modal(f, &self.state),
                    ActiveModal::Diff => render_diff_modal(f, ""),
                    ActiveModal::Logs => render_log_modal(f, &self.state),
                    ActiveModal::Assets => render_asset_modal(f, &[]),
                    ActiveModal::Memory => render_memory_modal(f, ""),
                    ActiveModal::None => {}
                }
            })?;

            if self.state.should_quit {
                break;
            }

            // Check if a new pipeline run was triggered from within the TUI
            if let Some((target_url, goal_prompt)) = self.state.should_trigger_pipeline.take() {
                let bus = self.event_bus.clone();
                tokio::spawn(async move {
                    let settings = Settings::load().unwrap_or_default();
                    let mut orch = PipelineOrchestrator::new(settings, bus);
                    let _ = orch.run_redesign_pipeline(&target_url, &goal_prompt).await;
                });
            }

            tokio::select! {
                Some(Ok(crossterm_event)) = event_reader.next() => {
                    if let crossterm::event::Event::Key(key) = crossterm_event {
                        if key.kind == KeyEventKind::Press {
                            KeybindingHandler::handle_key(&mut self.state, key);
                        }
                    }
                }
                Ok(bus_event) = bus_receiver.recv() => {
                    self.state.handle_event(&bus_event);
                }
                _ = tokio::time::sleep(Duration::from_millis(50)) => {}
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
}
