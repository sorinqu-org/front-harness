use crate::tui::state::{FocusedPane, TuiState};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_dag_tree(f: &mut Frame, area: Rect, state: &TuiState) {
    let is_focused = state.focused_pane == FocusedPane::DagTree;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = state
        .dag_steps
        .iter()
        .map(|step| {
            let (status_color, symbol) = match step.status.as_str() {
                "DONE" => (Color::Green, "[V]"),
                "RUNNING" => (Color::Yellow, "[*]"),
                "FAILED" => (Color::Red, "[X]"),
                _ => (Color::DarkGray, "[ ]"),
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", symbol), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(step.name.clone(), Style::default().fg(Color::White)),
            ]);

            let detail = Line::from(vec![
                Span::raw("    "),
                Span::styled(step.detail.clone(), Style::default().fg(Color::Gray)),
            ]);

            ListItem::new(vec![line, detail, Line::raw("")])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Pipeline DAG ")
        .border_style(border_style);

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
