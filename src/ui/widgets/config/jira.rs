use crate::config::model::JiraConfig;
use crate::state::jira_config::{JiraConfigEditor, JiraField};
use crate::ui::styles::edit_border_style;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use tui_text_field::TextField;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut JiraConfigEditor,
    config: Option<&JiraConfig>,
) {
    if let Some(form) = &state.form {
        render_inline_edit(frame, area, form);
    } else {
        render_values_section(frame, area, config);
    }
}

fn render_values_section(frame: &mut Frame, area: Rect, config: Option<&JiraConfig>) {
    let (url, email, token) = match config {
        Some(c) => (c.url.as_str(), c.email.as_str(), c.token.as_str()),
        None => ("", "", ""),
    };

    // Mask the token in display mode — shown in full only when editing.
    let token_display = if token.is_empty() {
        ""
    } else {
        "••••••••"
    };

    let lines = vec![
        display_line("URL  ", url),
        display_line("Email", email),
        display_line("Token", token_display),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn display_line(label: &str, value: &str) -> Line<'static> {
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

fn render_inline_edit(
    frame: &mut Frame,
    area: Rect,
    form: &crate::state::jira_config::JiraConfigForm,
) {
    let block = Block::bordered()
        .title(" Edit Jira Config ")
        .border_style(edit_border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let af = form.active_field;
    let lines = vec![
        field_line("URL  ", form.url.value(), af == JiraField::Url),
        Line::from(""),
        field_line("Email", form.email.value(), af == JiraField::Email),
        Line::from(""),
        field_line("Token", form.token.value(), af == JiraField::Token),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);

    // Place the terminal cursor on the active field.
    // Format is "{label}: {value}" where label is 5 chars → prefix = 7 chars.
    let row: u16 = match form.active_field {
        JiraField::Url => 0,
        JiraField::Email => 2,
        JiraField::Token => 4,
    };
    let char_offset = char_offset_to_cursor(form.active_field());
    frame.set_cursor_position((inner.x + 7 + char_offset, inner.y + row));
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
        Span::styled(format!("{label}: "), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

/// Returns the number of display columns from the start of the value to the cursor position.
fn char_offset_to_cursor(field: &TextField) -> u16 {
    field.value()[..field.cursor()].chars().count() as u16
}
