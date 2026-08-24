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
    let area = centered_rect(88, 88, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" FrontHarness Design Studio & Pipeline Launcher [Tab: Switch | Space: Toggle Skill | Enter: Launch | Esc: Close] ")
        .style(Style::default().bg(Color::Rgb(15, 15, 18)))
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // 0: Target Source
            Constraint::Length(3), // 1: Workspace Dir
            Constraint::Length(3), // 2: Business Goal
            Constraint::Length(3), // 3: Design Style Directives
            Constraint::Length(3), // 4: References
            Constraint::Length(4), // 5: Skills Matrix Checkboxes
            Constraint::Length(3), // 6: Action buttons & hints
        ])
        .split(block.inner(area));

    // Field 0: Target Source (URL or Local Directory)
    let f0_active = state.run_input_focus == 0;
    let f0_border = if f0_active { Color::Yellow } else { Color::DarkGray };
    let f0_val = if f0_active { format!("{}_", state.run_target_source) } else { state.run_target_source.clone() };
    let p0 = Paragraph::new(Line::from(vec![Span::styled(format!(" {}", f0_val), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" 1. Target Source (URL or Local Site Folder) ", Style::default().fg(if f0_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(f0_border)));

    // Field 1: Workspace Output Directory
    let f1_active = state.run_input_focus == 1;
    let f1_border = if f1_active { Color::Yellow } else { Color::DarkGray };
    let f1_val = if f1_active { format!("{}_", state.run_workspace_dir) } else { state.run_workspace_dir.clone() };
    let p1 = Paragraph::new(Line::from(vec![Span::styled(format!(" {}", f1_val), Style::default().fg(Color::White))]))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" 2. Workspace Output Directory ", Style::default().fg(if f1_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(f1_border)));

    // Field 2: Business Goal & Requirements
    let f2_active = state.run_input_focus == 2;
    let f2_border = if f2_active { Color::Yellow } else { Color::DarkGray };
    let f2_val = if f2_active { format!("{}_", state.run_goal_prompt) } else { state.run_goal_prompt.clone() };
    let p2 = Paragraph::new(Line::from(vec![Span::styled(format!(" {}", f2_val), Style::default().fg(Color::White))]))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" 3. Business Goal & Requirements ", Style::default().fg(if f2_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(f2_border)));

    // Field 3: Design Style Directives (Enforced Aesthetic)
    let f3_active = state.run_input_focus == 3;
    let f3_border = if f3_active { Color::Yellow } else { Color::DarkGray };
    let f3_val = if f3_active { format!("{}_", state.run_design_style) } else { state.run_design_style.clone() };
    let p3 = Paragraph::new(Line::from(vec![Span::styled(format!(" {}", f3_val), Style::default().fg(Color::LightCyan))]))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" 4. Design Style Directives (Palette, Fonts, Aesthetic) ", Style::default().fg(if f3_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(f3_border)));

    // Field 4: References & Inspiration
    let f4_active = state.run_input_focus == 4;
    let f4_border = if f4_active { Color::Yellow } else { Color::DarkGray };
    let f4_val = if f4_active { format!("{}_", state.run_references) } else { state.run_references.clone() };
    let p4 = Paragraph::new(Line::from(vec![Span::styled(format!(" {}", f4_val), Style::default().fg(Color::White))]))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" 5. Design References (URLs or local image paths) ", Style::default().fg(if f4_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(f4_border)));

    // Field 5: Skills Matrix Selection Menu
    let f5_active = state.run_input_focus == 5;
    let f5_border = if f5_active { Color::Yellow } else { Color::DarkGray };
    let mut skill_spans = Vec::new();
    for (idx, skill) in state.run_skills.iter().enumerate() {
        let is_cursor = f5_active && idx == state.run_skills_cursor;
        let checkbox = if skill.enabled { "[X] " } else { "[ ] " };
        
        let style = if is_cursor {
            Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else if skill.enabled {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        skill_spans.push(Span::styled(format!("{}{}", checkbox, skill.name), style));
        skill_spans.push(Span::raw("   "));
    }

    let skills_p = Paragraph::new(vec![
        Line::from(skill_spans),
        Line::from(vec![
            Span::styled("  Navigate: ", Style::default().fg(Color::Gray)),
            Span::styled("Left/Right (h/l)", Style::default().fg(Color::Cyan)),
            Span::styled(" | Toggle: ", Style::default().fg(Color::Gray)),
            Span::styled("Space", Style::default().fg(Color::Yellow)),
        ]),
    ]).block(Block::default().borders(Borders::ALL).title(Span::styled(" 6. Active Skills Matrix (Mandatory Rules) ", Style::default().fg(if f5_active { Color::Yellow } else { Color::Gray }).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(f5_border)));

    // Field 6: Action Footer
    let footer_line = Line::from(vec![
        Span::styled(" [Tab / Shift+Tab] ", Style::default().fg(Color::Cyan)),
        Span::raw("Switch Inputs | "),
        Span::styled(" [Enter] ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" Launch Multi-Agent Redesign Pipeline ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(" [Esc] ", Style::default().fg(Color::DarkGray)),
        Span::raw("Cancel"),
    ]);
    let footer_p = Paragraph::new(footer_line).block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));

    f.render_widget(block, area);
    f.render_widget(p0, inner[0]);
    f.render_widget(p1, inner[1]);
    f.render_widget(p2, inner[2]);
    f.render_widget(p3, inner[3]);
    f.render_widget(p4, inner[4]);
    f.render_widget(skills_p, inner[5]);
    f.render_widget(footer_p, inner[6]);
}
