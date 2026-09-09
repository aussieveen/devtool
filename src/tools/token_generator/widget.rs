use super::state::{Focus, Token, TokenGenerator};
use crate::config::model::ServiceConfig;
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

    let (service_idx, _audience_idx, _env_idx) = state.selected_service_audience_env();
    let service_config = &service_configs[service_idx];
    let show_audiences = service_config.audiences.len() > 1;

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if show_audiences {
            vec![
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
        } else {
            vec![Constraint::Percentage(50), Constraint::Percentage(50)]
        })
        .split(area);

    let service_focused = matches!(state.focus, Focus::Service);
    let audience_focused = matches!(state.focus, Focus::Audience);
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

    frame.render_stateful_widget(services, columns[0], &mut state.service_list_state);

    let env_area = if show_audiences {
        let audiences = List::new(
            service_config
                .audiences
                .iter()
                .map(|a| ListItem::new(a.clone())),
        )
        .highlight_style(selection_highlight())
        .block(
            Block::new()
                .borders(Borders::ALL)
                .title(" Audiences ")
                .border_style(block_style(audience_focused)),
        );

        frame.render_stateful_widget(audiences, columns[1], &mut state.audience_list_state);
        columns[2]
    } else {
        columns[1]
    };

    let audience_idx = state.audience_list_state.selected().unwrap_or_default();

    let environments = List::new(service_config.credentials.iter().enumerate().map(
        |(env_idx, c)| {
            let token = &state.tokens[service_idx][audience_idx][env_idx];
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

    frame.render_stateful_widget(environments, env_area, &mut state.env_list_state);
}
