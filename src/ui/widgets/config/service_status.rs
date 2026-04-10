use crate::config::model::ServiceStatusConfig;
use crate::state::service_status_config::{AddServicePopup, PopupField, ServiceStatusConfigEditor};
use crate::ui::styles::{key_desc_style, key_style, list_style};
use crate::utils::popup::popup_area;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table, Wrap};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut ServiceStatusConfigEditor,
    config: &[ServiceStatusConfig],
) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let list_area = vertical[0];
    let action_area = vertical[1];

    let selected = state.table_state.selected();

    if config.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(
                "No services yet — press [a] to add one.",
            ))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
            list_area,
        );
    } else {
        // Table with truncated URLs for readability
        let header = Row::new(["Name", "Staging", "Preproduction", "Production", "Repo"]);

        let rows: Vec<Row> = config
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let is_active = selected.is_none_or(|i| i == idx);
                Row::new([
                    Cell::from(s.name.clone()),
                    Cell::from(truncate(&s.staging, 20)),
                    Cell::from(truncate(&s.preproduction, 20)),
                    Cell::from(truncate(&s.production, 20)),
                    Cell::from(truncate(&s.repo, 20)),
                ])
                .style(list_style(is_active))
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
        .block(Block::default());

        frame.render_stateful_widget(table, list_area, &mut state.table_state);
    }

    // Action bar
    let key = key_style();
    let desc = key_desc_style();
    let mut actions = vec![
        Span::styled("[a]", key),
        Span::styled(" Add service  ", desc),
    ];
    if selected.is_some() && !config.is_empty() {
        actions.push(Span::styled("[e]", key));
        actions.push(Span::styled(" Edit selected  ", desc));
        actions.push(Span::styled("[x]", key));
        actions.push(Span::styled(" Remove selected  ", desc));
    }
    actions.push(Span::styled("[←]", key));
    actions.push(Span::styled(" Back to config  ", desc));

    frame.render_widget(
        Paragraph::new(Line::from(actions)).wrap(Wrap { trim: false }),
        action_area,
    );

    // Popup
    if let Some(popup) = &state.popup {
        render_popup(frame, frame.area(), popup);
    }
}

fn render_popup(frame: &mut Frame, area: Rect, popup: &AddServicePopup) {
    let popup_rect = popup_area(area, 55, 14);
    frame.render_widget(Clear, popup_rect);

    let title = if popup.edit_index.is_some() {
        " Edit Service "
    } else {
        " Add Service "
    };

    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let lines = vec![
        field_line("Name        ", &popup.name, popup.active_field == PopupField::Name),
        Line::from(""),
        field_line("Staging URL ", &popup.staging, popup.active_field == PopupField::Staging),
        Line::from(""),
        field_line("Preprod URL ", &popup.preprod, popup.active_field == PopupField::Preprod),
        Line::from(""),
        field_line("Prod URL    ", &popup.prod, popup.active_field == PopupField::Prod),
        Line::from(""),
        field_line("Repo URL    ", &popup.repo, popup.active_field == PopupField::Repo),
        Line::from(""),
        Line::from(Span::styled(
            "  [enter] Save   [esc] Cancel   [tab] Next field   [shift+tab] Prev field",
            Style::default().fg(Color::Gray),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
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
        Span::styled(format!("  {label}: "), label_style),
        Span::styled(format!("{value}{cursor}"), value_style),
    ])
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
