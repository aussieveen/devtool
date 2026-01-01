use crate::state::git_compare::{GitCompare, LinkStatus};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use std::fmt::format;
use crossterm::style::style;
use crate::ui::styles::list_style;

pub fn render(frame: &mut Frame, area: Rect, state: &mut GitCompare) {
    // let services = List::new(state.services.iter().map(|s| ListItem::new(s.name.clone())))
    //     .highlight_style(ratatui::style::Style::default().reversed())
    //     .highlight_symbol(">> ")
    //     .repeat_highlight_symbol(true);
    //
    // frame.render_stateful_widget(services, chunks[0], &mut state.list_state);

    let selected_service_idx = state.list_state.selected();

    let service = if selected_service_idx.is_some() {
        state.services.get(selected_service_idx.unwrap())
    }else{
        None
    };

    let (status_text, service_action_text) = if service.is_some() {
        match service.unwrap().link_status() {
            LinkStatus::Errored => ("Error when attempting to get diff. Do you need to be on the VPN?",""),
            LinkStatus::Diff => ("Preproduction and Production are on different commits", "[o] to Open in browser [c] to Copy the url"),
            LinkStatus::NoDiff => ("Preproduction and Production are on the same commit", ""),
            _ => ("",""),
        }
    }else{
        ("","")
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

    for (_, service) in state.services.iter().enumerate() {
        let preprod_commit = service.preprod.value();
        let prod_commit = service.prod.value();
        let service_colour = if(preprod_commit.unwrap_or("NO VALUE") == prod_commit.unwrap_or("NO VALUE")){
            Color::Green
        } else {
            Color::Red
        };

        columns[0].push((service.name.clone(), service_colour));
        columns[1].push((
            service
                .preprod
                .short_value()
                .unwrap_or("No commit found".to_string()),
            service_colour,
        ));
        columns[2].push((
            service
                .prod
                .short_value()
                .unwrap_or("No commit found".to_string()),
            service_colour,
        ));
    }

    // ── 5. Render commit grid columns ───────────────────────────────────────────────
    for (col_idx, col_area) in grid_cols.iter().enumerate() {
        let lines: Vec<Line> = columns[col_idx]
            .iter()
            .enumerate()
            .map(|(row_idx, (hash, status_color))| {
                let line_style = list_style(
                    (selected_service_idx.is_some() && selected_service_idx.unwrap() == row_idx) || selected_service_idx.is_none()
                );

                Line::from(vec![
                    // status stripe
                    Span::styled("▍", Style::default().bg(*status_color)),
                    Span::raw(" "),
                    Span::styled(hash.clone(), line_style),
                ])
            })
            .collect();

        // let line_style = if selected_service_idx.is_some() && selected_service_idx.unwrap() == col_idx{
        //     Style::default().fg(Color::DarkGray)
        // }else{
        //     Style::default()
        // };

        let column = Paragraph::new(lines)
            .block(Block::default())
            .alignment(Alignment::Left);

        frame.render_widget(column, *col_area);
    }

    frame.render_widget(Paragraph::new(status_text), status_area);
    let action_text = format!("{} {}", "[s] to scan the services", service_action_text );
    frame.render_widget(Paragraph::new(action_text).wrap(Wrap { trim: false }), action_area);
}
