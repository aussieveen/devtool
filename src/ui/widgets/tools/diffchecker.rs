use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};
use crate::state::diffchecker::{Commit, DiffChecker};

pub fn render(frame: &mut Frame, area: Rect, state: &mut DiffChecker){
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Percentage(50),
        ])
        .split(area);

    let services = List::new(
        state.services.iter().map(|s| ListItem::new(s.config.name.clone()))
    )
        .highlight_style(ratatui::style::Style::default().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(
        services,
        chunks[0],
        &mut state.state,
    );

    let service_idx = state.state.selected().unwrap();
    let service = &state.services[service_idx];
    let text = if service.preprod_fetched() && service.prod_fetched(){
        "Link available: [o] to Open in browser"
    } else {
        ""
    };

    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), chunks[1]);
}