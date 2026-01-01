use std::fmt::format;
use crate::state::git_compare::{GitCompare, LinkStatus};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect, state: &mut GitCompare) {
    // let services = List::new(state.services.iter().map(|s| ListItem::new(s.name.clone())))
    //     .highlight_style(ratatui::style::Style::default().reversed())
    //     .highlight_symbol(">> ")
    //     .repeat_highlight_symbol(true);
    //
    // frame.render_stateful_widget(services, chunks[0], &mut state.list_state);

    let service_idx = state.list_state.selected().unwrap();
    let service = &state.services[service_idx];

    let text = match service.link_status() {
        LinkStatus::Fetching => "Retrieving commit references",
        LinkStatus::Errored => "Error when attempting to get diff. Do you need to be on the VPN?",
        LinkStatus::Diff => "Link available: [o] to Open in browser, [c] to Copy the url",
        LinkStatus::NoDiff => "Preprod and Prod are on the same commit",
        LinkStatus::Missing => "[Return] to retrieve commit",
    };

    // ── 1. Split into header row + grid ─────────────────────────────────────────────
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // column headers
            Constraint::Min(0),    // commit grid
            Constraint::Length(2), // status code instructions
            Constraint::Length(2), // additional actions
        ])
        .split(area);

    let header_area = vertical[0];
    let grid_area = vertical[1];
    let status_area = vertical[2];
    let action_area = vertical[3];

    // ── 2. Define shared 25% column layout ──────────────────────────────────────────
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ]);

    let header_cols = columns.split(header_area);
    let grid_cols = columns.split(grid_area);

    // ── 3. Render column headers ────────────────────────────────────────────────────
    let headers = ["Service", "Preproduction", "Production"];

    for (i, title) in headers.iter().enumerate() {
        let header = Paragraph::new(*title)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(header, header_cols[i]);
    }

    // ── 4. Example commit data per column ───────────────────────────────────────────
    let mut columns: Vec<Vec<(String, Color)>> = vec![
        Vec::new(), // Service
        Vec::new(), // Preprod
        Vec::new(), // Prod
    ];

    for (service_idx, service) in state.services.iter().enumerate() {
        columns[0].push((
            service.name.clone(),
            Color::Gray
        ));
        columns[1].push((
            service.preprod.short_value().unwrap_or("No commit found".to_string()),
            Color::Green
        ));
        columns[2].push((
            service.prod.short_value().unwrap_or("No commit found".to_string()),
            Color::Red
        ));
    }

    // ── 5. Render commit grid columns ───────────────────────────────────────────────
    for (col_idx, col_area) in grid_cols.iter().enumerate() {
        let lines: Vec<Line> = columns[col_idx]
            .iter()
            .map(|(hash, status_color)| {
                Line::from(vec![
                    // status stripe
                    Span::styled("▍", Style::default().bg(*status_color)),
                    Span::raw(" "),
                    Span::styled(hash.clone(), Style::default().fg(Color::Gray)),
                ])
            })
            .collect();

        let column = Paragraph::new(lines)
            .block(Block::default())
            .alignment(Alignment::Left);

        frame.render_widget(column, *col_area);
    }

    frame.render_widget(
        Paragraph::new("STATUS TEXT"),
        status_area,
    );
    frame.render_widget(Paragraph::new("[s] to scan the services"), action_area);
}
