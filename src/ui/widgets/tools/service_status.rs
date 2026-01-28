use crate::state::service_status::{CommitRefStatus, ServiceStatus};
use crate::ui::styles::list_style;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect, state: &mut ServiceStatus) {
    const ALL_MATCH: Color = Color::Green;
    const NONE_MATCH: Color = Color::Red;
    const PREPROD_PROD_MATCH: Color = Color::Blue;
    const STAGING_PREPROD_MATCH: Color = Color::Yellow;

    let selected_service_idx = state.list_state.selected();

    // ── 1. Split into header row + grid ─────────────────────────────────────────────
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // column headers
            Constraint::Min(0),    // commit grid
            Constraint::Length(2), // color legend
            Constraint::Length(2), // additional actions
        ])
        .split(area);

    let header_area = vertical[0];
    let grid_area = vertical[1];
    let legend_area = vertical[2];
    let action_area = vertical[3];

    // ── 2. Define shared 25% column layout ──────────────────────────────────────────
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ]);

    let header_cols = columns.split(header_area);
    let grid_cols = columns.split(grid_area);

    // ── 3. Render column headers ────────────────────────────────────────────────────
    let headers = ["Service", "Staging", "Preproduction", "Production"];

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
        Vec::new(), // Staging
        Vec::new(), // Preprod
        Vec::new(), // Prod
    ];

    for service in state.services.iter() {
        let (service_color, staging_color, preprod_color, prod_color) = match service
            .commit_ref_status()
        {
            CommitRefStatus::NothingMatches => (NONE_MATCH, Color::Red, Color::Red, Color::Red),
            CommitRefStatus::AllMatches => (ALL_MATCH, Color::Green, Color::Green, Color::Green),
            CommitRefStatus::StagingPreprodMatch => (
                STAGING_PREPROD_MATCH,
                Color::Green,
                Color::Green,
                Color::Red,
            ),
            CommitRefStatus::PreprodProdMatch => (
                PREPROD_PROD_MATCH,
                Color::Yellow,
                Color::Green,
                Color::Green,
            ),
            CommitRefStatus::CommitMissing => (Color::Red, Color::Red, Color::Red, Color::Red),
        };

        let no_commit: &str = "Unable to get commit";

        columns[0].push((service.name.clone(), service_color));
        columns[1].push((
            service
                .staging
                .short_value()
                .unwrap_or(no_commit.to_string()),
            staging_color,
        ));
        columns[2].push((
            service
                .preprod
                .short_value()
                .unwrap_or(no_commit.to_string()),
            preprod_color,
        ));
        columns[3].push((
            service.prod.short_value().unwrap_or(no_commit.to_string()),
            prod_color,
        ));
    }

    // ── 5. Render commit grid columns ───────────────────────────────────────────────
    for (col_idx, col_area) in grid_cols.iter().enumerate() {
        let lines: Vec<Line> = columns[col_idx]
            .iter()
            .enumerate()
            .map(|(row_idx, (hash, status_color))| {
                let line_style = list_style(
                    (selected_service_idx.is_some() && selected_service_idx.unwrap() == row_idx)
                        || selected_service_idx.is_none(),
                );

                Line::from(vec![
                    // status stripe
                    Span::styled("▍", Style::default().bg(*status_color)),
                    Span::raw(" "),
                    Span::styled(hash.clone(), line_style),
                ])
            })
            .collect();

        let column = Paragraph::new(lines)
            .block(Block::default())
            .alignment(Alignment::Left);

        frame.render_widget(column, *col_area);
    }

    // ── 5. Render legend,status and action rows ────────────────────────────────────

    let service_action_text = if let Some(service) = state
        .list_state
        .selected()
        .and_then(|idx| state.services.get(idx))
    {
        match service.commit_ref_status() {
            CommitRefStatus::StagingPreprodMatch | CommitRefStatus::NothingMatches => {
                "[o] to Open in browser [c] to Copy the url"
            }
            _ => "",
        }
    } else {
        ""
    };

    let action_text = format!("{} {} {}", "[s] to scan the services", service_action_text, if(selected_service_idx.is_some()){selected_service_idx.unwrap()}else{0});
    frame.render_widget(
        Paragraph::new(action_text).wrap(Wrap { trim: false }),
        action_area,
    );

    let legend_text = Line::from(vec![
        Span::styled("▍", Style::default().bg(ALL_MATCH)),
        Span::raw(" Up to date  "),
        Span::styled("▍", Style::default().bg(PREPROD_PROD_MATCH)),
        Span::raw(" New version in deployment pipeline  "),
        Span::styled("▍", Style::default().bg(STAGING_PREPROD_MATCH)),
        Span::raw(" Pending production deployment  "),
        Span::styled("▍", Style::default().bg(NONE_MATCH)),
        Span::raw(" Requires maintenance  "),
    ]);

    frame.render_widget(
        Paragraph::new(legend_text).wrap(Wrap { trim: false }),
        legend_area,
    );
}
