use crate::tui::state::TuiState;
use crate::tui::widgets::help_modal::centered_rect;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_log_modal(f: &mut Frame, state: &TuiState) {
    let area = centered_rect(85, 75, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Event Bus & Engine Logs ")
        .style(Style::default().bg(Color::Rgb(15, 15, 15)))
        .border_style(Style::default().fg(Color::Yellow));

    let content = state.logs.join("\n");
    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}
