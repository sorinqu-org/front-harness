use crate::tui::state::{FocusedPane, TuiState};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_stream_view(f: &mut Frame, area: Rect, state: &TuiState) {
    let is_focused = state.focused_pane == FocusedPane::StreamBuffer;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let content = if state.stream_buffer.is_empty() {
        "Awaiting pipeline execution events..."
    } else {
        &state.stream_buffer
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Live Agent Generation Stream ")
        .border_style(border_style);

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset as u16, 0));

    f.render_widget(paragraph, area);
}
