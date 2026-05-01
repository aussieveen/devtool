use crate::config::model::{Auth0Config, ServiceConfig};
use crate::state::token_generator_config::{
    ActiveEdit, Auth0Field, ConfigFocus, ServiceField, TokenGeneratorConfigEditor,
};
use crate::ui::styles::{block_style, edit_border_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};
use tui_text_field::TextField;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut TokenGeneratorConfigEditor,
    auth0: &Auth0Config,
    services: &[ServiceConfig],
) {
    let auth0_editing = matches!(&state.form, Some(ActiveEdit::Auth0(_)));

    // Auth0 section height: 7 for display (4 lines + 1 blank + 2 borders),
    // 11 for inline edit (4 fields + 4 separators + 1 hint + 2 borders).
    let auth0_height = if auth0_editing { 11 } else { 7 };

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(auth0_height), Constraint::Min(0)])
        .split(area);

    let auth0_area = vertical[0];
    let services_area = vertical[1];

    if let Some(ActiveEdit::Auth0(p)) = &state.form {
        let p = p.clone();
        render_auth0_inline(
            frame,
            auth0_area,
            &p,
            state.config_focus == ConfigFocus::Auth0,
        );
        render_services_section(frame, services_area, state, services);
    } else if let Some(ActiveEdit::Service(p)) = &state.form {
        let p = p.clone();
        render_auth0_section(frame, auth0_area, auth0, false);
        render_service_inline(frame, services_area, &p);
    } else {
        render_auth0_section(
            frame,
            auth0_area,
            auth0,
            state.config_focus == ConfigFocus::Auth0,
        );
        render_services_section(frame, services_area, state, services);
    }
}

fn render_auth0_section(frame: &mut Frame, area: Rect, auth0: &Auth0Config, focused: bool) {
    let block = Block::bordered()
        .title(" Auth0 Endpoints ")
        .border_style(block_style(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        auth0_display_line("Local      ", &auth0.local),
        auth0_display_line("Staging    ", &auth0.staging),
        auth0_display_line("Preprod    ", &auth0.preproduction),
        auth0_display_line("Production ", &auth0.production),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn auth0_display_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), Style::default().fg(Color::Gray)),
        Span::styled(
            if value.is_empty() {
                "(not set)".to_string()
            } else {
                value.to_string()
            },
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
        .border_style(block_style(services_focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);

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
        .map(|s| {
            Row::new([
                Cell::from(s.name.clone()),
                Cell::from(truncate(&s.audience, 30)),
                Cell::from(s.credentials.len().to_string()),
            ])
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
    .row_highlight_style(selection_highlight())
    .block(Block::default());

    frame.render_stateful_widget(table, inner, &mut state.table_state);
}

// ── Auth0 inline edit ─────────────────────────────────────────────────────────

fn render_auth0_inline(
    frame: &mut Frame,
    area: Rect,
    form: &crate::state::token_generator_config::Auth0Form,
    focused: bool,
) {
    let block = Block::bordered()
        .title(" Auth0 Endpoints ")
        .border_style(if focused {
            edit_border_style()
        } else {
            block_style(false)
        });
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

    if focused {
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
}

// ── Service inline edit ───────────────────────────────────────────────────────

fn render_service_inline(
    frame: &mut Frame,
    area: Rect,
    form: &crate::state::token_generator_config::ServiceForm,
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
        field_line("Audience  ", form.audience.value(), af == ServiceField::Audience),
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
