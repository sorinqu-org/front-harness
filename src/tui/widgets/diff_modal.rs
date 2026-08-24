use crate::tui::widgets::help_modal::centered_rect;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_diff_modal(f: &mut Frame, diff_content: &str) {
    let area = centered_rect(80, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Unified Code Diff ")
        .style(Style::default().bg(Color::Rgb(15, 15, 15)))
        .border_style(Style::default().fg(Color::Green));

    let content = if diff_content.is_empty() {
        "No pending code diffs to inspect."
    } else {
        diff_content
    };

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}
