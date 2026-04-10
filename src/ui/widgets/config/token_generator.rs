use crate::config::model::{Auth0Config, ServiceConfig};
use crate::state::token_generator_config::{
    ActivePopup, Auth0Field, ConfigFocus, ServiceField, TokenGeneratorConfigEditor,
};
use crate::ui::styles::{block_style, key_desc_style, key_style, list_style};
use crate::utils::popup::popup_area;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table, Wrap};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGeneratorConfigEditor,
    auth0: &Auth0Config,
    services: &[ServiceConfig],
) {
    // Split into: auth0 section (top), services section (middle), action bar (bottom)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // auth0 section
            Constraint::Min(0),     // services table
            Constraint::Length(2),  // action bar
        ])
        .split(area);

    let auth0_area = vertical[0];
    let services_area = vertical[1];
    let action_area = vertical[2];

    render_auth0_section(frame, auth0_area, auth0, state.config_focus == ConfigFocus::Auth0);
    render_services_section(frame, services_area, state, services);
    render_action_bar(
        frame,
        action_area,
        state.config_focus,
        state.table_state.selected(),
        services,
    );

    // Popups
    if let Some(popup) = &state.popup {
        match popup {
            ActivePopup::Auth0(p) => render_auth0_popup(frame, frame.area(), p),
            ActivePopup::Service(p) => render_service_popup(frame, frame.area(), p),
        }
    }
}

fn render_auth0_section(frame: &mut Frame, area: Rect, auth0: &Auth0Config, focused: bool) {
    let block = Block::bordered()
        .title(" Auth0 Endpoints ")
        .title_alignment(Alignment::Center)
        .border_style(block_style(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        auth0_display_line("Local      ", &auth0.local),
        auth0_display_line("Staging    ", &auth0.staging),
        auth0_display_line("Preprod    ", &auth0.preproduction),
        auth0_display_line("Production ", &auth0.production),
    ];
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

fn auth0_display_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            if value.is_empty() { "(not set)".to_string() } else { value.to_string() },
            if value.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            },
        ),
    ])
}

fn render_services_section(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGeneratorConfigEditor,
    services: &[ServiceConfig],
) {
    let services_focused = state.config_focus == ConfigFocus::Services;
    let block = Block::bordered()
        .title(" Services ")
        .title_alignment(Alignment::Center)
        .border_style(block_style(services_focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let selected = state.table_state.selected();

    if services.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No services yet — press [a] to add one."))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let header = Row::new(["Name", "Audience", "Envs"]);
    let rows: Vec<Row> = services
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let is_active = selected.is_none_or(|i| i == idx);
            Row::new([
                Cell::from(s.name.clone()),
                Cell::from(truncate(&s.audience, 30)),
                Cell::from(s.credentials.len().to_string()),
            ])
            .style(list_style(is_active))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(60),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .block(Block::default());

    frame.render_stateful_widget(table, inner, &mut state.table_state);
}

fn render_action_bar(
    frame: &mut Frame,
    area: Rect,
    config_focus: ConfigFocus,
    selected: Option<usize>,
    services: &[ServiceConfig],
) {
    let key = key_style();
    let desc = key_desc_style();
    let mut actions = vec![
        Span::styled("[a]", key),
        Span::styled(" Add service  ", desc),
    ];
    match config_focus {
        ConfigFocus::Auth0 => {
            actions.insert(0, Span::styled(" Edit Auth0  ", desc));
            actions.insert(0, Span::styled("[e]", key));
        }
        ConfigFocus::Services => {
            if selected.is_some() && !services.is_empty() {
                actions.push(Span::styled("[e]", key));
                actions.push(Span::styled(" Edit selected  ", desc));
                actions.push(Span::styled("[x]", key));
                actions.push(Span::styled(" Remove selected  ", desc));
            }
        }
    }
    actions.push(Span::styled("[←]", key));
    actions.push(Span::styled(" Back to config  ", desc));

    frame.render_widget(
        Paragraph::new(Line::from(actions)).wrap(Wrap { trim: false }),
        area,
    );
}

// ── Auth0 popup ───────────────────────────────────────────────────────────────

fn render_auth0_popup(
    frame: &mut Frame,
    area: Rect,
    popup: &crate::state::token_generator_config::Auth0Popup,
) {
    let popup_rect = popup_area(area, 55, 12);
    frame.render_widget(Clear, popup_rect);

    let block = Block::bordered()
        .title(" Edit Auth0 Endpoints ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let lines = vec![
        field_line("Local      ", &popup.local, popup.active_field == Auth0Field::Local),
        Line::from(""),
        field_line("Staging    ", &popup.staging, popup.active_field == Auth0Field::Staging),
        Line::from(""),
        field_line("Preprod    ", &popup.preprod, popup.active_field == Auth0Field::Preprod),
        Line::from(""),
        field_line("Production ", &popup.prod, popup.active_field == Auth0Field::Prod),
        Line::from(""),
        hint_line(),
    ];

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ── Service popup ─────────────────────────────────────────────────────────────

fn render_service_popup(
    frame: &mut Frame,
    area: Rect,
    popup: &crate::state::token_generator_config::ServicePopup,
) {
    let popup_rect = popup_area(area, 58, 28);
    frame.render_widget(Clear, popup_rect);

    let title = if popup.edit_index.is_some() {
        " Edit Service "
    } else {
        " Add Service "
    };

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let af = popup.active_field;
    let lines = vec![
        field_line("Name      ", &popup.name, af == ServiceField::Name),
        Line::from(""),
        field_line("Audience  ", &popup.audience, af == ServiceField::Audience),
        Line::from(""),
        divider_line("Local"),
        field_line("Client ID ", &popup.local_id, af == ServiceField::LocalClientId),
        field_line("Client Sec", &popup.local_secret, af == ServiceField::LocalClientSecret),
        Line::from(""),
        divider_line("Staging"),
        field_line("Client ID ", &popup.staging_id, af == ServiceField::StagingClientId),
        field_line("Client Sec", &popup.staging_secret, af == ServiceField::StagingClientSecret),
        Line::from(""),
        divider_line("Preprod"),
        field_line("Client ID ", &popup.preprod_id, af == ServiceField::PreprodClientId),
        field_line("Client Sec", &popup.preprod_secret, af == ServiceField::PreprodClientSecret),
        Line::from(""),
        divider_line("Production"),
        field_line("Client ID ", &popup.prod_id, af == ServiceField::ProdClientId),
        field_line("Client Sec", &popup.prod_secret, af == ServiceField::ProdClientSecret),
        Line::from(""),
        hint_line(),
    ];

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn field_line(label: &str, value: &str, active: bool) -> Line<'static> {
    let label_style = if active {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let value_style = if active {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let cursor = if active { "_" } else { "" };
    Line::from(vec![
        Span::styled(format!("  {label}: "), label_style),
        Span::styled(format!("{value}{cursor}"), value_style),
    ])
}

fn divider_line(label: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ── {label} "),
        Style::default().fg(Color::Gray),
    ))
}

fn hint_line() -> Line<'static> {
    Line::from(Span::styled(
        "  [enter] Save   [esc] Cancel   [↑/↓] Navigate",
        Style::default().fg(Color::Gray),
    ))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
