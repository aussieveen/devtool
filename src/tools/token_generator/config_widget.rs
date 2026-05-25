use super::config_editor::{ActiveEdit, Auth0Field, ServiceField, TokenGeneratorConfigEditor};
use crate::config::model::{Auth0Config, ServiceConfig};
use crate::ui::styles::{edit_border_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Row, Table, Wrap};
use ratatui::widgets::Paragraph;
use tui_text_field::TextField;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGeneratorConfigEditor,
    auth0: &Auth0Config,
    services: &[ServiceConfig],
) {
    match &state.form {
        Some(ActiveEdit::Auth0(p)) => {
            let p = p.clone();
            render_auth0_inline(frame, area, &p);
        }
        Some(ActiveEdit::Service(p)) => {
            let p = p.clone();
            render_service_inline(frame, area, &p);
        }
        None => render_unified_table(frame, area, state, auth0, services),
    }
}

// ── Unified table (browse mode) ───────────────────────────────────────────────

fn render_unified_table(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGeneratorConfigEditor,
    auth0: &Auth0Config,
    services: &[ServiceConfig],
) {
    let auth0_status = auth0_endpoint_status(auth0);
    let header = Row::new(["Name", "Details", "Envs"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let auth0_row = Row::new([
        Cell::from("Auth0 Endpoints").style(Style::default().fg(Color::Cyan)),
        Cell::from(auth0_status).style(Style::default().fg(Color::DarkGray)),
        Cell::from("—"),
    ]);

    let service_rows: Vec<Row> = services
        .iter()
        .map(|s| {
            Row::new([
                Cell::from(s.name.clone()),
                Cell::from(truncate(&s.audience, 40)),
                Cell::from(s.credentials.len().to_string()),
            ])
        })
        .collect();

    let mut all_rows = vec![auth0_row];
    if services.is_empty() {
        all_rows.push(
            Row::new([
                Cell::from("(no services yet — press [a] to add one)")
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(""),
                Cell::from(""),
            ]),
        );
    } else {
        all_rows.extend(service_rows);
    }

    let table = Table::new(
        all_rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(65),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .row_highlight_style(selection_highlight())
    .block(Block::bordered());

    frame.render_stateful_widget(table, area, &mut state.table_state);
}

fn auth0_endpoint_status(auth0: &Auth0Config) -> String {
    let configured = [
        &auth0.local,
        &auth0.staging,
        &auth0.preproduction,
        &auth0.production,
    ]
    .iter()
    .filter(|s| !s.is_empty())
    .count();
    match configured {
        0 => "(not set)".to_string(),
        4 => "all endpoints configured".to_string(),
        n => format!("{n}/4 endpoints configured"),
    }
}

// ── Auth0 inline edit ─────────────────────────────────────────────────────────

fn render_auth0_inline(
    frame: &mut Frame,
    area: Rect,
    form: &crate::tools::token_generator::config_editor::Auth0Form,
) {
    let block = Block::bordered()
        .title(" Auth0 Endpoints ")
        .border_style(edit_border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        field_line(
            "Local      ",
            form.local.value(),
            form.active_field == Auth0Field::Local,
        ),
        Line::from(""),
        field_line(
            "Staging    ",
            form.staging.value(),
            form.active_field == Auth0Field::Staging,
        ),
        Line::from(""),
        field_line(
            "Preprod    ",
            form.preprod.value(),
            form.active_field == Auth0Field::Preprod,
        ),
        Line::from(""),
        field_line(
            "Production ",
            form.prod.value(),
            form.active_field == Auth0Field::Prod,
        ),
        Line::from(""),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);

    // Format is "  {label}: {value}" where label is 11 chars → prefix = 15 chars.
    let row: u16 = match form.active_field {
        Auth0Field::Local => 0,
        Auth0Field::Staging => 2,
        Auth0Field::Preprod => 4,
        Auth0Field::Prod => 6,
    };
    let char_offset = char_offset_to_cursor(form.active_field());
    frame.set_cursor_position((inner.x + 15 + char_offset, inner.y + row));
}

// ── Service inline edit ───────────────────────────────────────────────────────

fn render_service_inline(
    frame: &mut Frame,
    area: Rect,
    form: &crate::tools::token_generator::config_editor::ServiceForm,
) {
    let title = if form.edit_index.is_some() {
        " Edit Service "
    } else {
        " Add Service "
    };

    let block = Block::bordered()
        .title(title)
        .border_style(edit_border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let af = form.active_field;
    let lines = vec![
        field_line("Name      ", form.name.value(), af == ServiceField::Name),
        Line::from(""),
        field_line(
            "Audience  ",
            form.audience.value(),
            af == ServiceField::Audience,
        ),
        Line::from(""),
        divider_line("Local"),
        field_line(
            "Client ID ",
            form.local_id.value(),
            af == ServiceField::LocalClientId,
        ),
        field_line(
            "Client Sec",
            form.local_secret.value(),
            af == ServiceField::LocalClientSecret,
        ),
        Line::from(""),
        divider_line("Staging"),
        field_line(
            "Client ID ",
            form.staging_id.value(),
            af == ServiceField::StagingClientId,
        ),
        field_line(
            "Client Sec",
            form.staging_secret.value(),
            af == ServiceField::StagingClientSecret,
        ),
        Line::from(""),
        divider_line("Preprod"),
        field_line(
            "Client ID ",
            form.preprod_id.value(),
            af == ServiceField::PreprodClientId,
        ),
        field_line(
            "Client Sec",
            form.preprod_secret.value(),
            af == ServiceField::PreprodClientSecret,
        ),
        Line::from(""),
        divider_line("Production"),
        field_line(
            "Client ID ",
            form.prod_id.value(),
            af == ServiceField::ProdClientId,
        ),
        field_line(
            "Client Sec",
            form.prod_secret.value(),
            af == ServiceField::ProdClientSecret,
        ),
        Line::from(""),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);

    // Format is "  {label}: {value}" where label is 10 chars → prefix = 14 chars.
    // Row indices in the lines vec above:
    // Name=0, blank=1, Audience=2, blank=3, "Local"=4, LocalId=5, LocalSec=6,
    // blank=7, "Staging"=8, StagingId=9, StagingSec=10, blank=11, "Preprod"=12,
    // PreprodId=13, PreprodSec=14, blank=15, "Production"=16, ProdId=17, ProdSec=18
    let row: u16 = match form.active_field {
        ServiceField::Name => 0,
        ServiceField::Audience => 2,
        ServiceField::LocalClientId => 5,
        ServiceField::LocalClientSecret => 6,
        ServiceField::StagingClientId => 9,
        ServiceField::StagingClientSecret => 10,
        ServiceField::PreprodClientId => 13,
        ServiceField::PreprodClientSecret => 14,
        ServiceField::ProdClientId => 17,
        ServiceField::ProdClientSecret => 18,
    };
    let char_offset = char_offset_to_cursor(form.active_field());
    frame.set_cursor_position((inner.x + 14 + char_offset, inner.y + row));
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn field_line(label: &str, value: &str, active: bool) -> Line<'static> {
    let label_style = if active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let value_style = if active {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    Line::from(vec![
        Span::styled(format!("  {label}: "), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

fn char_offset_to_cursor(field: &TextField) -> u16 {
    field.value()[..field.cursor()].chars().count() as u16
}

fn divider_line(label: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ── {label} "),
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
