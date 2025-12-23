use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use crate::state::diffchecker::DiffChecker;

pub fn render(frame: &mut Frame, area: Rect, state: &mut DiffChecker){
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Percentage(50),
        ])
        .split(area);
    let test: String = state.services.iter().map(|s| s.config.name.clone()).collect();

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

    frame.render_widget(Paragraph::new(test), chunks[1]);
}