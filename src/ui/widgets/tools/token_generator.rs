use crate::state::token_generator::{Focus, Token, TokenGenerator};
use crate::ui::styles::list_style;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{List, ListItem, Paragraph};
use std::cmp::max;
use strum::EnumCount;
use crate::environment::Environment;

pub fn render(frame: &mut Frame, area: Rect, state: &mut TokenGenerator) {
    let vertical_break = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(max(state.services.capacity() as u16, Environment::COUNT as u16)),
            Constraint::Percentage(99),
        ])
        .split(area);

    let inner_horizonal = Layout::default()
        .spacing(1)
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_break[0]);

    let services = List::new(state.services.iter().map(|s| ListItem::new(s.name.clone())))
        .style(list_style(matches!(state.focus, Focus::Service)))
        .highlight_style(ratatui::style::Style::default().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(services, inner_horizonal[0], &mut state.service_list_state);

    let service_idx = state.service_list_state.selected().unwrap();
    let env_idx = state.env_list_state.selected().unwrap();

    let service = &state.services[service_idx];

    let environments = List::new(service.credentials.iter().map(|c| {
        let token = service.tokens.get(&c.env).unwrap();
        let prefix = match *token {
            Token::Generated(_) => "[✓]",
            Token::Error(_) => "[x]",
            _ => "[ ]",
        };
        ListItem::new(format!("{} {}", prefix, c.env.as_str()))
    }))
    .style(list_style(matches!(state.focus, Focus::Env)))
    .highlight_style(ratatui::style::Style::default().reversed())
    .highlight_symbol(">> ")
    .repeat_highlight_symbol(true);

    frame.render_stateful_widget(environments, inner_horizonal[1], &mut state.env_list_state);

    let env = &service.credentials[env_idx].env;

    let token = service.tokens.get(env).unwrap();

    let text = match *token {
        Token::NotGenerated => "[Return] to generate token",
        Token::Fetching => "Generating token",
        Token::Generated(_) => "Token available: [c] to Copy the token value",
        Token::Error(_) => &*format!(
            "{}: {}",
            "Error when attempting to get the token",
            token.value().unwrap()
        ),
    };

    frame.render_widget(Paragraph::new(text), vertical_break[1]);
}
