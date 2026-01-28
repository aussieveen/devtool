use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};
use crate::state::jira::Jira;
use crate::ui::styles::list_style;

pub fn render(frame: &mut Frame, area: Rect, state: &mut Option<Jira>) {
    if state.is_none(){
        return;
    }

    let jira_state = state.as_mut().unwrap();
    let selected_ticket = jira_state.list_state.selected();

    // ── 1. Split into header row + grid ─────────────────────────────────────────────
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // tickets
            Constraint::Length(2), // additional actions
        ])
        .split(area);

    let ticket_area = vertical[0];

    let list_items:Vec<ListItem> = jira_state.tickets
        .iter()
        .enumerate()
        .map(|(index, ticket)| {
            let line_style = list_style(
                (selected_ticket.is_some() && selected_ticket.unwrap() == index)
                    || selected_ticket.is_none(),
            );
            let mut lines: Vec<Line> = Vec::new();
            lines.push(
                Line::from(format!("{}", if(selected_ticket.is_some()){selected_ticket.unwrap()}else{0}))
            );
            lines.push(
                Line::from(format!("{} - {}", ticket.id, ticket.title))
            );
            lines.push(
                Line::from(format!("{} {}", ticket.assignee, ticket.status))
            );
            lines.push(Line::from(""));
            ListItem::from(lines).style(line_style)
        })
        .collect();

    frame.render_stateful_widget(
        List::new(list_items).block(Block::default()),
        ticket_area,
        &mut jira_state.list_state
    );

    let action_area = vertical[1];
    let ticket_action_text = ""; // WILL BE ADDED TO LATER AS STATES ARE MANAGED

    let action_text = format!("{} {}", "[a] to add ticket", ticket_action_text);
    frame.render_widget(
        Paragraph::new(action_text).wrap(Wrap { trim: false }),
        action_area
    );
}
