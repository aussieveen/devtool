use crate::state::jira::Jira;
use crate::ui::styles::{key_desc_style, key_style, list_style};
use crate::utils::popup::popup_area;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Span;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect, state: &mut Jira) {
    let selected_ticket = state.list_state.selected();
    let new_ticket_pop_up = state.new_ticket_popup;
    let new_ticket_id = state.new_ticket_id.clone().unwrap_or_default();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // tickets
            Constraint::Length(2), // additional actions
        ])
        .split(area);

    let ticket_area = vertical[0];

    let list_items: Vec<ListItem> = state
        .tickets
        .iter()
        .enumerate()
        .map(|(index, ticket)| {
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
            ListItem::from(lines).style(list_style(selected_ticket.is_none_or(|i| i == index)))
        })
        .collect();

    frame.render_stateful_widget(
        List::new(list_items).block(Block::default()),
        ticket_area,
        &mut state.list_state,
    );

    let action_area = vertical[1];

    let key = key_style();
    let desc = key_desc_style();

    let mut action_text = vec![
        Span::styled("[a]", key),
        Span::styled(" to add ticket  ", desc),
    ];

    if selected_ticket.is_some() {
        action_text.push(Span::styled("[x]", key));
        action_text.push(Span::styled(" to remove ticket  ", desc));
        action_text.push(Span::styled("[shift + ↑ ↓]", key));
        action_text.push(Span::styled(" to move tickets  ", desc));
    }

    frame.render_widget(
        Paragraph::new(Line::from(action_text)).wrap(Wrap { trim: false }),
        action_area,
    );

    if new_ticket_pop_up {
        let block = Block::bordered().title("Add Jira Ticket");
        let paragraph = Paragraph::new(new_ticket_id)
            .block(block)
            .alignment(Alignment::Center);

        let area = popup_area(frame.area(), 20, 3);
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
}
