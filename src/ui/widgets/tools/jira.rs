use crate::state::jira::Jira;
use crate::ui::styles::list_style;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::text::{Line};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect, state: &mut Option<Jira>) {
    if state.is_none() {
        return;
    }

    let jira_state = state.as_mut().unwrap();
    let selected_ticket = jira_state.list_state.selected();
    let new_ticket_pop_up = jira_state.new_ticket_popup;
    let new_ticket_id = jira_state.new_ticket_id.clone().unwrap_or_default();

    // ── 1. Split into header row + grid ─────────────────────────────────────────────
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // tickets
            Constraint::Length(2), // additional actions
        ])
        .split(area);

    let ticket_area = vertical[0];

    let list_items: Vec<ListItem> = jira_state
        .tickets
        .iter()
        .enumerate()
        .map(|(index, ticket)| {
            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(format!("{} - {}", ticket.id, ticket.title)));
            lines.push(Line::from(format!("{} {}", ticket.assignee, ticket.status)));
            lines.push(Line::from(""));
            ListItem::from(lines).style(list_style(
                (selected_ticket.is_some() && selected_ticket.unwrap() == index)
                    || selected_ticket.is_none(),
            ))
        })
        .collect();

    frame.render_stateful_widget(
        List::new(list_items).block(Block::default()),
        ticket_area,
        &mut jira_state.list_state,
    );

    let action_area = vertical[1];
    let ticket_action_text = match selected_ticket {
        Some(_) => "[x] to remove ticket [shift + ↑ ↓] to move tickets",
        None => "",
    };

    let action_text = format!("{} {}", "[a] to add ticket", ticket_action_text);
    frame.render_widget(
        Paragraph::new(action_text).wrap(Wrap { trim: false }),
        action_area,
    );

    if new_ticket_pop_up {
        let block = Block::bordered().title("Add Jira Ticket");
        let paragraph = Paragraph::new(new_ticket_id)
            .block(block)
            .alignment(Alignment::Center);

        let area = popup_area(frame.area(), 20);
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
}

fn popup_area(area: Rect, percent_x: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
