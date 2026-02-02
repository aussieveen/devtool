use crate::config::ServiceConfig;
use crate::state::token_generator::{Focus, Token, TokenGenerator};
use crate::ui::styles::list_style;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{List, ListItem, Paragraph};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGenerator,
    service_configs: &[ServiceConfig],
) {
    let vertical_break = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let inner_horizonal = Layout::default()
        .spacing(1)
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_break[0]);

    let services = List::new(
        service_configs
            .iter()
            .map(|s| ListItem::new(s.name.clone())),
    )
    .style(list_style(matches!(state.focus, Focus::Service)))
    .highlight_style(ratatui::style::Style::default().reversed())
    .highlight_symbol(">> ")
    .repeat_highlight_symbol(true);

    frame.render_stateful_widget(services, inner_horizonal[0], &mut state.service_list_state);

    let (service_idx, _env_idx) = state.get_selected_service_env();

    let service_config = &service_configs[service_idx];

    let environments = List::new(service_config.credentials.iter().enumerate().map(
        |(env_idx, c)| {
            let token = &state.tokens[service_idx][env_idx];
            let prefix = match token {
                Token::Ready(_) => "[✓]",
                Token::Error(_) => "[x]",
                _ => "[ ]",
            };
            ListItem::new(format!("{} {}", prefix, c.env.as_str()))
        },
    ))
    .style(list_style(matches!(state.focus, Focus::Env)))
    .highlight_style(ratatui::style::Style::default().reversed())
    .highlight_symbol(">> ")
    .repeat_highlight_symbol(true);

    frame.render_stateful_widget(environments, inner_horizonal[1], &mut state.env_list_state);

    let text = match state.get_token_for_selected_service_env() {
        Token::Idle => "[Return] to generate token",
        Token::Requesting => "Generating token",
        Token::Ready(_) => "Token available: [c] to Copy the token value",
        Token::Error(e) => &*format!("{}: {}", "Error when attempting to get the token", e),
    };

    frame.render_widget(Paragraph::new(text), vertical_break[1]);
}
