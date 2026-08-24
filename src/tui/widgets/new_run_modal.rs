use crate::tui::state::TuiState;
use crate::tui::widgets::help_modal::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_new_run_modal(f: &mut Frame, state: &TuiState) {
    let area = centered_rect(75, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Start New Redesign Pipeline [Enter: Launch | Tab: Switch Input | Esc: Cancel] ")
        .style(Style::default().bg(Color::Rgb(15, 15, 18)))
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(block.inner(area));

    // 1. Target URL Input
    let (url_border, url_label_style) = if state.run_input_focus == 0 {
        (Color::Yellow, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        (Color::DarkGray, Style::default().fg(Color::Gray))
    };

    let url_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" 1. Target Website URL ", url_label_style))
        .border_style(Style::default().fg(url_border));

    let url_text = if state.run_input_focus == 0 {
        format!("{}_", state.run_target_url)
    } else if state.run_target_url.is_empty() {
        "https://example.com/".to_string()
    } else {
        state.run_target_url.clone()
    };
    let url_p = Paragraph::new(Line::from(vec![
        Span::styled(format!("  {}", url_text), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ])).block(url_block);

    // 2. Goal / Prompt Input
    let (goal_border, goal_label_style) = if state.run_input_focus == 1 {
        (Color::Yellow, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        (Color::DarkGray, Style::default().fg(Color::Gray))
    };

    let goal_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" 2. Redesign Prompt / Business Goal ", goal_label_style))
        .border_style(Style::default().fg(goal_border));

    let goal_text = if state.run_input_focus == 1 {
        format!("{}_", state.run_goal_prompt)
    } else if state.run_goal_prompt.is_empty() {
        "Enter design instructions, target style, animations, conversion goals...".to_string()
    } else {
        state.run_goal_prompt.clone()
    };
    let goal_p = Paragraph::new(Line::from(vec![
        Span::styled(format!("  {}", goal_text), Style::default().fg(Color::White)),
    ])).block(goal_block);

    // 3. Hints & Info
    let hints = vec![
        Line::from(vec![
            Span::styled("Tips: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("For redesigns, enter a full URL (e.g. https://as-chelyabinsk.ru/)."),
        ]),
        Line::from(vec![
            Span::raw("      The pipeline will crawl DOM/CSS, extract images, research patterns, and generate modern code."),
        ]),
    ];
    let hints_p = Paragraph::new(hints);

    // 4. Action Bar
    let action_line = Line::from(vec![
        Span::styled(" [Tab] Switch Field ", Style::default().fg(Color::Cyan)),
        Span::styled(" [Enter] Launch Multi-Agent Pipeline ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" [Esc] Cancel ", Style::default().fg(Color::DarkGray)),
    ]);
    let action_block = Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray));
    let action_p = Paragraph::new(action_line).block(action_block);

    f.render_widget(block, area);
    f.render_widget(url_p, inner[0]);
    f.render_widget(goal_p, inner[1]);
    f.render_widget(hints_p, inner[2]);
    f.render_widget(action_p, inner[3]);
}
