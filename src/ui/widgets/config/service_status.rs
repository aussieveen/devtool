use crate::config::model::ServiceStatusConfig;
use crate::state::service_status_config::{AddServiceForm, FormField, ServiceStatusConfigEditor};
use crate::ui::styles::{edit_border_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};
use tui_text_field::TextField;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut ServiceStatusConfigEditor,
    config: &[ServiceStatusConfig],
) {
    if let Some(form) = &state.form {
        let form = form.clone();
        render_inline_edit(frame, area, &form);
    } else {
        render_table(frame, area, state, config);
    }
}

fn render_table(
    frame: &mut Frame,
    area: Rect,
    state: &mut ServiceStatusConfigEditor,
    config: &[ServiceStatusConfig],
) {
    if config.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No services yet — press [a] to add one."))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let header = Row::new(["Name", "Staging", "Preproduction", "Production", "Repo"]);
    let rows: Vec<Row> = config
        .iter()
        .map(|s| {
            Row::new([
                Cell::from(s.name.clone()),
                Cell::from(truncate(&s.staging, 20)),
                Cell::from(truncate(&s.preproduction, 20)),
                Cell::from(truncate(&s.production, 20)),
                Cell::from(truncate(&s.repo, 20)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(18),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .row_highlight_style(selection_highlight())
    .block(Block::default());

    frame.render_stateful_widget(table, area, &mut state.table_state);
}

fn render_inline_edit(frame: &mut Frame, area: Rect, form: &AddServiceForm) {
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

    let lines = vec![
        field_line(
            "Name        ",
            form.name.value(),
            form.active_field == FormField::Name,
        ),
        Line::from(""),
        field_line(
            "Staging URL ",
            form.staging.value(),
            form.active_field == FormField::Staging,
        ),
        Line::from(""),
        field_line(
            "Preprod URL ",
            form.preprod.value(),
            form.active_field == FormField::Preprod,
        ),
        Line::from(""),
        field_line(
            "Prod URL    ",
            form.prod.value(),
            form.active_field == FormField::Prod,
        ),
        Line::from(""),
        field_line(
            "Repo URL    ",
            form.repo.value(),
            form.active_field == FormField::Repo,
        ),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);

    // Place the terminal cursor on the active field.
    // Format is "  {label}: {value}" where label is 12 chars → prefix = 16 chars.
    let row: u16 = match form.active_field {
        FormField::Name => 0,
        FormField::Staging => 2,
        FormField::Preprod => 4,
        FormField::Prod => 6,
        FormField::Repo => 8,
    };
    let field = form.active_field();
    let char_offset = char_offset_to_cursor(field);
    frame.set_cursor_position((inner.x + 16 + char_offset, inner.y + row));
}

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

/// Returns the number of display columns from the start of the value to the cursor position.
fn char_offset_to_cursor(field: &TextField) -> u16 {
    field.value()[..field.cursor()].chars().count() as u16
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
