use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{List, ListItem, Paragraph, Wrap};
use crate::state::diffchecker::{DiffChecker, LinkStatus};

pub fn render(frame: &mut Frame, area: Rect, state: &mut DiffChecker){
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),
            Constraint::Percentage(99),
        ])
        .split(area);

    let services = List::new(
        state.services.iter().map(|s| ListItem::new(s.name.clone()))
    )
        .highlight_style(ratatui::style::Style::default().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(
        services,
        chunks[0],
        &mut state.list_state,
    );

    let service_idx = state.list_state.selected().unwrap();
    let service = &state.services[service_idx];

    let text = match service.link_status() {
        LinkStatus::Fetching => "Retrieving commit references",
        LinkStatus::Errored => "Error when attempting to get diff. Do you need to be on the VPN?",
        LinkStatus::Diff => "Link available: [o] to Open in browser, [c] to Copy the url",
        LinkStatus::NoDiff => "Preprod and Prod are on the same commit",
        LinkStatus::Missing => "[Return] to retrieve commit",
    };

    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), chunks[1]);
}