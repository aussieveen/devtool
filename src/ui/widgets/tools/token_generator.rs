use crate::config::model::ServiceConfig;
use crate::state::token_generator::{Focus, Token, TokenGenerator};
use crate::ui::styles::{block_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGenerator,
    service_configs: &[ServiceConfig],
) {
    if service_configs.is_empty() {
        use ratatui::layout::Alignment;
        use ratatui::style::Style;
        use ratatui::widgets::Paragraph;
        frame.render_widget(
            Paragraph::new(
                "No token generator services configured — press [2] then Enter on Token Generator to configure.",
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(ratatui::style::Color::DarkGray)),
            area,
        );
        return;
    }

    const READY_COLOR: Color = Color::Green;
    const ERROR_COLOR: Color = Color::Red;
    const REQUESTING_COLOR: Color = Color::Yellow;

    let inner_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

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
            .border_style(block_style(env_focused)),
    );

    frame.render_stateful_widget(environments, inner_horizontal[1], &mut state.env_list_state);
}
