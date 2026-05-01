use crate::state::jira::Jira;
use crate::ui::styles::{edit_border_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Span;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use tui_text_field::TextField;

pub fn render(frame: &mut Frame, area: Rect, state: &mut Jira) {
    let adding_ticket = state.adding_ticket;

    // When adding a ticket, reserve 3 rows for the inline input; otherwise use full area.
    let ticket_area = if adding_ticket {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);
        let input_area = vertical[1];
        render_add_ticket_input(frame, input_area, &state.new_ticket_id);
        vertical[0]
    } else {
        area
    };

    let list_items: Vec<ListItem> = state
        .tickets
        .iter()
        .map(|ticket| {
            let status_color = match ticket.status.to_lowercase().as_str() {
                s if s.contains("complete") => Color::Green,
                s if s.contains("release") => Color::Magenta,
                s if s.contains("in test") => Color::LightCyan,
                s if s.contains("testing") => Color::Cyan,
                s if s.contains("review") => Color::Yellow,
                s if s.contains("progress") => Color::Blue,
                s if s.contains("development") => Color::Gray,
                s if s.contains("failed") => Color::Red,
                _ => Color::DarkGray,
            };

            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(vec![
                Span::styled(ticket.id.clone(), Style::default().fg(Color::Cyan)),
                Span::raw(format!(" - {}", ticket.title)),
            ]));
            lines.push(Line::from(vec![
                Span::styled(ticket.status.clone(), Style::default().fg(status_color)),
                Span::styled(
                    format!("  @{}", ticket.assignee),
                    Style::default().fg(Color::LightBlue),
                ),
            ]));
            lines.push(Line::from(""));
            ListItem::from(lines)
        })
        .collect();

    frame.render_stateful_widget(
        List::new(list_items)
            .highlight_style(selection_highlight())
            .block(Block::default()),
        ticket_area,
        &mut state.list_state,
    );
}

fn render_add_ticket_input(frame: &mut Frame, area: Rect, field: &TextField) {
    let block = Block::bordered()
        .title(" Add Jira Ticket ")
        .border_style(edit_border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            field.value().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )])),
        inner,
    );

    let char_offset = field.value()[..field.cursor()].chars().count() as u16;
    frame.set_cursor_position((inner.x + char_offset, inner.y));
}
