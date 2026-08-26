use crate::agents::PipelineOrchestrator;
use crate::core::event_bus::EventBus;
use crate::tui::keybindings::KeybindingHandler;
use crate::tui::state::{ActiveModal, TuiState};
use crate::tui::widgets::{
    render_asset_modal, render_config_modal, render_dag_tree, render_diff_modal, render_help_modal,
    render_log_modal, render_memory_modal, render_new_run_modal, render_review_modal,
    render_statusline, render_stream_view,
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
                    ActiveModal::Review => render_review_modal(f, &self.state),
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

            // 1. Redesign Pipeline Trigger
            if let Some((target_source, goal_prompt, settings)) = self.state.should_trigger_pipeline.take() {
                let bus = self.event_bus.clone();
                tokio::spawn(async move {
                    let mut orch = PipelineOrchestrator::new(settings, bus.clone());
                    let res = orch.run_redesign_pipeline(&target_source, &goal_prompt).await;
                    if let Err(e) = res {
                        bus.emit_error("PipelineOrchestrator", &format!("Redesign pipeline failed: {}", e));
                    }
                });
            }

            // 2. Greenfield Pipeline Trigger
            if let Some((niche, goal_prompt, settings)) = self.state.should_trigger_greenfield.take() {
                let bus = self.event_bus.clone();
                tokio::spawn(async move {
                    let mut orch = PipelineOrchestrator::new(settings, bus.clone());
                    let res = orch.run_greenfield_pipeline(&niche, &goal_prompt).await;
                    if let Err(e) = res {
                        bus.emit_error("PipelineOrchestrator", &format!("Greenfield pipeline failed: {}", e));
                    }
                });
            }

            // 3. Refinement Iteration Trigger
            if let Some((critique, rating, settings)) = self.state.should_trigger_refinement.take() {
                let bus = self.event_bus.clone();
                tokio::spawn(async move {
                    let mut orch = PipelineOrchestrator::new(settings, bus.clone());
                    let res = orch.run_refinement_iteration(&critique, rating).await;
                    if let Err(e) = res {
                        bus.emit_error("PipelineOrchestrator", &format!("Refinement iteration failed: {}", e));
                    }
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
                res = bus_receiver.recv() => {
                    match res {
                        Ok(bus_event) => {
                            self.state.handle_event(&bus_event);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {}
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(30)) => {}
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
}
