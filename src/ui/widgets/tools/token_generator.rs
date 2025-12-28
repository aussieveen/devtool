use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{List, ListItem, Paragraph};
use crate::config::Credentials;
use crate::environment::Environment;
use crate::state::token_generator::{Focus, TokenGenerator};
use crate::ui::styles::list_style;

pub fn render(frame: &mut Frame, area: Rect, state: &mut TokenGenerator){
    let vertical_break = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Percentage(99),
        ])
        .split(area);

    let inner_horizonal = Layout::default()
        .spacing(1)
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .split(vertical_break[0]);

    let services = List::new(
        state.services.iter().map(|s| ListItem::new(s.name.clone()))
    )
        .style(list_style(matches!(state.focus, Focus::Service)))
        .highlight_style(ratatui::style::Style::default().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(
        services,
        inner_horizonal[0],
        &mut state.service_list_state,
    );

    let service_idx = state.service_list_state.selected().unwrap();



    let env = List::new(
        state.services[service_idx].credentials.iter().map(|c| ListItem::new(c.env.as_str()))
    )
        .style(list_style(matches!(state.focus, Focus::Env)))
        .highlight_style(ratatui::style::Style::default().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(
        env,
        inner_horizonal[1],
        &mut state.env_list_state,
    );

    frame.render_widget(Paragraph::new("TOKEN MESSAGING"), vertical_break[1]);

}