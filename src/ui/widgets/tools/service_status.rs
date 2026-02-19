use crate::config::model::ServiceStatus as ServiceStatusConfig;
use crate::state::service_status::{Commit, CommitRefStatus, ServiceStatus};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut ServiceStatus,
    config: &[ServiceStatusConfig],
) {
    const ALL_MATCH: Color = Color::Green;
    const NONE_MATCH: Color = Color::Red;
    const PREPROD_PROD_MATCH: Color = Color::Cyan;
    const STAGING_PREPROD_MATCH: Color = Color::Yellow;

    let commit_cell = |commit: &Commit, ok_color: Color| -> (String, Color) {
        match commit {
            Commit::Fetching => ("…".to_string(), Color::DarkGray),
            Commit::Empty => ("—".to_string(), Color::DarkGray),
            Commit::Error(_) => ("Error".to_string(), NONE_MATCH),
            Commit::Ok(_) => (commit.short_value().unwrap(), ok_color),
        }
    };

    let selected_service_idx = state.list_state.selected();

    // ── 1. Split into header row + grid ─────────────────────────────────────────────
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // column headers
            Constraint::Min(0),    // commit grid
            Constraint::Min(0),    // request errors
            Constraint::Length(2), // color legend
            Constraint::Length(2), // additional actions
        ])
        .split(area);

    let header_area = vertical[0];
    let grid_area = vertical[1];
    let error_area = vertical[2];
    let legend_area = vertical[3];
    let action_area = vertical[4];

    // ── 2. Define shared 25% column layout ──────────────────────────────────────────
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(23),
            Constraint::Percentage(23),
            Constraint::Percentage(24),
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

    for (service_idx, service) in state.services.iter().enumerate() {
        let (service_color, staging_ok, preprod_ok, prod_ok) = match service.commit_ref_status() {
            CommitRefStatus::NothingMatches => (NONE_MATCH, Color::Red, Color::Red, Color::Red),
            CommitRefStatus::AllMatches => (ALL_MATCH, Color::Green, Color::Green, Color::Green),
            CommitRefStatus::StagingPreprodMatch => {
                (STAGING_PREPROD_MATCH, Color::Green, Color::Green, Color::Red)
            }
            CommitRefStatus::PreprodProdMatch => {
                (PREPROD_PROD_MATCH, PREPROD_PROD_MATCH, Color::Green, Color::Green)
            }
            CommitRefStatus::CommitMissing => {
                (NONE_MATCH, Color::DarkGray, Color::DarkGray, Color::DarkGray)
            }
        };

        columns[0].push((config[service_idx].name.clone(), service_color));
        columns[1].push(commit_cell(&service.staging, staging_ok));
        columns[2].push(commit_cell(&service.preproduction, preprod_ok));
        columns[3].push(commit_cell(&service.production, prod_ok));
    }

    // ── 5. Render commit grid columns ───────────────────────────────────────────────
    for (col_idx, col_area) in grid_cols.iter().enumerate() {
        let lines: Vec<Line> = columns[col_idx]
            .iter()
            .enumerate()
            .map(|(row_idx, (text, color))| {
                let is_active = selected_service_idx.is_none()
                    || selected_service_idx == Some(row_idx);

                if col_idx == 0 {
                    // Service name column: status stripe + name
                    let stripe_color = if is_active { *color } else { Color::DarkGray };
                    let text_style = if is_active {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    Line::from(vec![
                        Span::styled("▍", Style::default().bg(stripe_color)),
                        Span::raw(" "),
                        Span::styled(text.clone(), text_style),
                    ])
                } else {
                    // Env columns: colored text only, no stripe
                    let text_color = if is_active { *color } else { Color::DarkGray };
                    Line::from(Span::styled(
                        text.clone(),
                        Style::default().fg(text_color),
                    ))
                }
            })
            .collect();

        let column = Paragraph::new(lines)
            .block(Block::default())
            .alignment(Alignment::Left);

        frame.render_widget(column, *col_area);
    }

    // ── Render errors
    if let Some(service_idx) = state.list_state.selected() {
        let mut lines: Vec<Line> = vec![];
        let service = &state.services[service_idx];
        let commits = vec![
            (&service.staging, "Staging"),
            (&service.preproduction, "Preproduction"),
            (&service.production, "Production"),
        ];
        for commit in commits {
            let (c, env) = commit;
            if let Some(error) = c.get_error() {
                lines.push(format!("{}: {}", env, error).into());
            }
        }
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), error_area);
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

    let action_text = format!("{} {}", "[s] to scan the services", service_action_text);
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
