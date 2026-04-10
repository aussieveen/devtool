use crate::config::model::JiraConfig;
use crate::state::jira_config::{JiraConfigEditor, JiraField};
use crate::ui::styles::edit_border_style;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

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
        field_line("URL  ", &form.url, af == JiraField::Url),
        Line::from(""),
        field_line("Email", &form.email, af == JiraField::Email),
        Line::from(""),
        field_line("Token", &form.token, af == JiraField::Token),
        Line::from(""),
        hint_line(),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
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
    let cursor = if active { "_" } else { "" };
    Line::from(vec![
        Span::styled(format!("{label}: "), label_style),
        Span::styled(format!("{value}{cursor}"), value_style),
    ])
}

fn hint_line() -> Line<'static> {
    Line::from(Span::styled(
        "[enter] Save   [esc] Cancel   [↑/↓] Navigate",
        Style::default().fg(Color::Gray),
    ))
}
