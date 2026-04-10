use crate::config::model::JiraConfig;
use crate::state::jira_config::{JiraConfigEditor, JiraField};
use crate::ui::styles::{block_style, key_desc_style, key_style};
use crate::utils::popup::popup_area;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut JiraConfigEditor,
    config: Option<&JiraConfig>,
) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    render_values_section(frame, vertical[0], config);
    render_action_bar(frame, vertical[1]);

    if let Some(popup) = &state.popup {
        render_popup(frame, frame.area(), popup);
    }
}

fn render_values_section(frame: &mut Frame, area: Rect, config: Option<&JiraConfig>) {
    let block = Block::bordered()
        .title(" Jira Connection ")
        .title_alignment(Alignment::Center)
        .border_style(block_style(true));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (url, email, token) = match config {
        Some(c) => (c.url.as_str(), c.email.as_str(), c.token.as_str()),
        None => ("", "", ""),
    };

    let lines = vec![
        display_line("URL  ", url),
        display_line("Email", email),
        display_line("Token", token),
    ];
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

fn display_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default().fg(Color::Gray),
        ),
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

fn render_action_bar(frame: &mut Frame, area: Rect) {
    let key = key_style();
    let desc = key_desc_style();
    let actions = vec![
        Span::styled("[e]", key),
        Span::styled(" Edit  ", desc),
        Span::styled("[←]", key),
        Span::styled(" Back to config  ", desc),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(actions)).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_popup(
    frame: &mut Frame,
    area: Rect,
    popup: &crate::state::jira_config::JiraConfigPopup,
) {
    let popup_rect = popup_area(area, 55, 11);
    frame.render_widget(Clear, popup_rect);

    let block = Block::bordered()
        .title(" Edit Jira Connection ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let af = popup.active_field;
    let lines = vec![
        field_line("URL  ", &popup.url, af == JiraField::Url),
        Line::from(""),
        field_line("Email", &popup.email, af == JiraField::Email),
        Line::from(""),
        field_line("Token", &popup.token, af == JiraField::Token),
        Line::from(""),
        hint_line(),
    ];

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

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
