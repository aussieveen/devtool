use crate::config::model::ServiceConfig;
use crate::state::token_generator::{Focus, Token, TokenGenerator};
use crate::ui::styles::{block_style, key_desc_style, key_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGenerator,
    service_configs: &[ServiceConfig],
) {
    const READY_COLOR: Color = Color::Green;
    const ERROR_COLOR: Color = Color::Red;
    const REQUESTING_COLOR: Color = Color::Yellow;

    let vertical_break = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let inner_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_break[0]);

    let service_focused = matches!(state.focus, Focus::Service);
    let env_focused = matches!(state.focus, Focus::Env);

    let services = List::new(
        service_configs
            .iter()
            .map(|s| ListItem::new(s.name.clone())),
    )
    .highlight_style(selection_highlight())
    .block(
        Block::new()
            .borders(Borders::ALL)
            .title(" Services ")
            .title_alignment(Alignment::Center)
            .border_style(block_style(service_focused)),
    );

    frame.render_stateful_widget(services, inner_horizontal[0], &mut state.service_list_state);

    let (service_idx, _env_idx) = state.get_selected_service_env();

    let service_config = &service_configs[service_idx];

    let environments = List::new(service_config.credentials.iter().enumerate().map(
        |(env_idx, c)| {
            let token = &state.tokens[service_idx][env_idx];
            let (prefix, prefix_style) = match token {
                Token::Ready(_) => ("[✓]", Style::default().fg(READY_COLOR)),
                Token::Error => ("[x]", Style::default().fg(ERROR_COLOR)),
                Token::Requesting => ("[…]", Style::default().fg(REQUESTING_COLOR)),
                _ => ("[ ]", Style::default()),
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::raw(format!(" {}", c.env)),
            ]))
        },
    ))
    .highlight_style(selection_highlight())
    .block(
        Block::new()
            .borders(Borders::ALL)
            .title(" Environments ")
            .title_alignment(Alignment::Center)
            .border_style(block_style(env_focused)),
    );

    frame.render_stateful_widget(environments, inner_horizontal[1], &mut state.env_list_state);

    let key = key_style();
    let desc = key_desc_style();

    let token_text = match state.get_token_for_selected_service_env() {
        Token::Idle => {
            vec![
                Span::styled("[Return]", key),
                Span::styled(" to generate token  ", desc),
            ]
        }
        Token::Requesting => {
            vec![Span::from("Generating token")]
        }
        Token::Ready(_) => {
            vec![
                Span::from("Token available: "),
                Span::styled("[c]", key),
                Span::styled(" to Copy the token value", desc),
            ]
        }
        Token::Error => {
            vec![Span::from("Error when attempting to get the token")]
        }
    };

    frame.render_widget(Paragraph::new(Line::from(token_text)), vertical_break[1]);
}
