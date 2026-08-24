use crate::tui::state::TuiState;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_statusline(f: &mut Frame, area: Rect, state: &TuiState) {
    let mode_style = Style::default()
        .bg(Color::Cyan)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let info_style = Style::default().bg(Color::DarkGray).fg(Color::White);
    let stat_style = Style::default().bg(Color::Rgb(30, 30, 30)).fg(Color::Yellow);

    let spans = vec![
        Span::styled(format!(" [{}] ", state.current_phase.to_uppercase()), mode_style),
        Span::styled(format!(" Model: {} ", state.model_name), info_style),
        Span::styled(format!(" Reasoning: {} ", state.reasoning_effort), info_style),
        Span::styled(format!(" DevServer: {} ", state.dev_server_status), stat_style),
        Span::styled(" [r: New Run | c: Config | d: Diff | l: Logs | m: Memory | ?: Help | q: Quit] ", Style::default().fg(Color::DarkGray)),
    ];

    let line = Line::from(spans);
    let widget = Paragraph::new(line);
    f.render_widget(widget, area);
}
