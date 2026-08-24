use crate::tui::widgets::help_modal::centered_rect;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_memory_modal(f: &mut Frame, memory_insights: &str) {
    let area = centered_rect(75, 65, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Long-Term Memory & User Feedback ")
        .style(Style::default().bg(Color::Rgb(15, 15, 15)))
        .border_style(Style::default().fg(Color::Blue));

    let text = if memory_insights.is_empty() {
        "No previous project memory records found. Insights will be logged after pipeline execution."
    } else {
        memory_insights
    };

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}
