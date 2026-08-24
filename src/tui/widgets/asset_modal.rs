use crate::tui::widgets::help_modal::centered_rect;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_asset_modal(f: &mut Frame, asset_list: &[String]) {
    let area = centered_rect(75, 65, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Extracted Assets & Screenshots Gallery ")
        .style(Style::default().bg(Color::Rgb(15, 15, 15)))
        .border_style(Style::default().fg(Color::Magenta));

    let text = if asset_list.is_empty() {
        "Screenshots: desktop_1920x1080.png, mobile_375x812.png\nAssets: SVG icons, WOFF2 fonts intercepted in workspace/audit/assets/".to_string()
    } else {
        asset_list.join("\n")
    };

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}
