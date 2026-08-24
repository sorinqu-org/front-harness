use crate::tui::state::TuiState;
use crate::tui::widgets::help_modal::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_config_modal(f: &mut Frame, state: &TuiState) {
    let area = centered_rect(82, 80, f.area());
    f.render_widget(Clear, area);

    let title = if state.is_editing_field {
        " FrontHarness Config Editor [EDIT MODE: Type new value & press Enter] "
    } else {
        " FrontHarness Config Editor [Space: Toggle/Cycle | Enter: Edit | s: Save | Esc: Exit] "
    };

    let border_color = if state.is_editing_field {
        Color::Yellow
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Rgb(15, 15, 18)))
        .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(state.config_fields.len() as u16 * 3),
            Constraint::Length(3),
        ])
        .split(block.inner(area));

    let mut lines = Vec::new();

    for (idx, field) in state.config_fields.iter().enumerate() {
        let is_selected = idx == state.config_selected_index;

        let display_val = if is_selected && state.is_editing_field {
            format!("{}_", state.config_edit_buffer)
        } else if field.is_secret && !field.value.is_empty() {
            if field.value.len() > 10 {
                format!("{}...{}", &field.value[..6], &field.value[field.value.len() - 4..])
            } else {
                "********".to_string()
            }
        } else if field.value.is_empty() {
            "<unset>".to_string()
        } else {
            field.value.clone()
        };

        let (prefix, label_style, val_style) = if is_selected {
            if state.is_editing_field {
                (
                    "> ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "> ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD),
                )
            }
        } else {
            (
                "  ",
                Style::default().fg(Color::Gray),
                Style::default().fg(Color::LightCyan),
            )
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<22}", field.label), label_style),
            Span::styled(format!(" [ {} ]", display_val), val_style),
        ]));

        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(&field.description, Style::default().fg(Color::Rgb(120, 120, 135))),
        ]));

        lines.push(Line::from(Span::raw("")));
    }

    let form_paragraph = Paragraph::new(lines);
    f.render_widget(block, area);
    f.render_widget(form_paragraph, inner[0]);

    // Bottom Help & Status Bar
    let status_text = if let Some(ref msg) = state.config_status_message {
        Line::from(vec![
            Span::styled(" STATUS: ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", msg), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])
    } else if state.is_editing_field {
        Line::from(vec![
            Span::styled(" EDITING ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" Type to modify value. Press Enter to confirm, Esc to discard.", Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" [j/k] Navigate ", Style::default().fg(Color::Cyan)),
            Span::styled(" [Space] Toggle/Cycle Options ", Style::default().fg(Color::Yellow)),
            Span::styled(" [Enter/i] Edit ", Style::default().fg(Color::White)),
            Span::styled(" [s] Save to Disk ", Style::default().fg(Color::Green)),
            Span::styled(" [Esc/q] Close ", Style::default().fg(Color::DarkGray)),
        ])
    };

    let bottom_block = Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray));
    let bottom_p = Paragraph::new(status_text).block(bottom_block);
    f.render_widget(bottom_p, inner[1]);
}
