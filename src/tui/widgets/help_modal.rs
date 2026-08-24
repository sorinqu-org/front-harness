use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_help_modal(f: &mut Frame) {
    let area = centered_rect(70, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" FrontHarness Shortcuts & Keybindings ")
        .style(Style::default().bg(Color::Rgb(20, 20, 20)))
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let text = vec![
        Line::from(vec![Span::styled("r / n", Style::default().fg(Color::Yellow)), Span::raw("             : Start New Pipeline Run (Input URL & Goal/Prompt)")]),
        Line::from(vec![Span::styled("c", Style::default().fg(Color::Yellow)), Span::raw("                 : Edit Configuration (API Keys, Models, Reasoning, Ports)")]),
        Line::from(vec![Span::styled("Tab / Shift+Tab", Style::default().fg(Color::Yellow)), Span::raw("   : Cycle focused pane (DAG / Stream / Logs)")]),
        Line::from(vec![Span::styled("j / k or Up/Down", Style::default().fg(Color::Yellow)), Span::raw(" : Scroll active pane buffer")]),
        Line::from(vec![Span::styled("d", Style::default().fg(Color::Yellow)), Span::raw("                 : View code diffs")]),
        Line::from(vec![Span::styled("l", Style::default().fg(Color::Yellow)), Span::raw("                 : View full logs modal")]),
        Line::from(vec![Span::styled("a", Style::default().fg(Color::Yellow)), Span::raw("                 : View captured screenshots & assets")]),
        Line::from(vec![Span::styled("m", Style::default().fg(Color::Yellow)), Span::raw("                 : View Long-Term Memory")]),
        Line::from(vec![Span::styled("?", Style::default().fg(Color::Yellow)), Span::raw("                 : Toggle this help screen")]),
        Line::from(vec![Span::styled("Esc / q", Style::default().fg(Color::Yellow)), Span::raw("           : Close modal / Quit application")]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
