use crate::tui::state::TuiState;
use crate::tui::widgets::help_modal::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_review_modal(f: &mut Frame, state: &TuiState) {
    let area = centered_rect(80, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" FrontHarness Project Review & Iterative Refinement [Keys: 1-5 Rating | Tab/Down: Next | Enter: Apply] ")
        .style(Style::default().bg(Color::Rgb(15, 15, 20)))
        .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4), // 0: Star Rating
            Constraint::Length(7), // 1: Critique / What to fix
            Constraint::Length(3), // 2: Apply fixes button
            Constraint::Length(3), // 3: Save & exit button
        ])
        .split(block.inner(area));

    // Section 0: Star Rating
    let f0_active = state.review_focus == 0;
    let mut star_spans = Vec::new();
    star_spans.push(Span::raw("  "));
    for score in 1..=5 {
        let is_selected = state.review_rating == score;
        let style = if is_selected {
            Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        star_spans.push(Span::styled(format!(" [ ★ {} Star{} ] ", score, if score > 1 { "s" } else { "" }), style));
        star_spans.push(Span::raw(" "));
    }
    star_spans.push(Span::styled(" (Press 1..5 or Left/Right)", Style::default().fg(if f0_active { Color::Yellow } else { Color::DarkGray })));

    let p_rating = Paragraph::new(vec![
        Line::from(star_spans),
        Line::from(vec![
            Span::styled("  Current evaluation: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} / 5 stars", state.review_rating), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
    ]).block(Block::default().borders(Borders::ALL).title(Span::styled(" 1. Evaluation Score (1-5) ", Style::default().fg(if f0_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(if f0_active { Color::Yellow } else { Color::DarkGray })));

    // Section 1: Critique & Refinement Request
    let f1_active = state.review_focus == 1;
    let f1_border = if f1_active { Color::Yellow } else { Color::DarkGray };
    let f1_val = if f1_active { format!("{}_", state.review_critique) } else { state.review_critique.clone() };
    let p_critique = Paragraph::new(Line::from(vec![Span::styled(format!(" {}", f1_val), Style::default().fg(Color::White))]))
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" 2. Critique & What to Fix (Prompt for Next Multi-Agent Turn) ", Style::default().fg(if f1_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(f1_border)));

    // Section 2: Apply Iterative Fixes Button
    let f2_active = state.review_focus == 2;
    let btn1_style = if f2_active {
        Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };
    let p_btn1 = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(" [ >>> APPLY ITERATIVE REFINEMENT & RE-GENERATE <<< ] ", btn1_style),
        Span::raw("   "),
        Span::styled("(Triggers surgical CoderAgent iteration)", Style::default().fg(Color::DarkGray)),
    ])).block(Block::default().borders(Borders::NONE));

    // Section 3: Save & Finish Button
    let f3_active = state.review_focus == 3;
    let btn2_style = if f3_active {
        Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };
    let p_btn2 = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(" [ Save Rating & Feedback to Memory (Exit) ] ", btn2_style),
        Span::raw("   "),
        Span::styled("[Esc] Close without saving", Style::default().fg(Color::DarkGray)),
    ])).block(Block::default().borders(Borders::NONE));

    f.render_widget(block, area);
    f.render_widget(p_rating, inner[0]);
    f.render_widget(p_critique, inner[1]);
    f.render_widget(p_btn1, inner[2]);
    f.render_widget(p_btn2, inner[3]);
}
