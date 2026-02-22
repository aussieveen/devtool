use crate::config::model::ServiceStatus as ServiceStatusConfig;
use crate::state::service_status::{Commit, CommitRefStatus, ServiceStatus};
use crate::ui::styles::{key_desc_style, key_style, row_style};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};

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

    let selected_service_idx = state.table_state.selected();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // table
            Constraint::Min(0),    // request errors
            Constraint::Length(2), // color legend
            Constraint::Length(2), // additional actions
        ])
        .split(area);

    let table_area = vertical[0];
    let error_area = vertical[1];
    let legend_area = vertical[2];
    let action_area = vertical[3];

    let headers = Row::new(vec!["Service", "Staging", "Preproduction", "Production"]);
    let rows: Vec<Row> = state
        .services
        .iter()
        .enumerate()
        .map(|(service_idx, service)| {
            let (service_color, staging_ok, preprod_ok, prod_ok) = match service.commit_ref_status()
            {
                CommitRefStatus::NothingMatches => (NONE_MATCH, Color::Red, Color::Red, Color::Red),
                CommitRefStatus::AllMatches => {
                    (ALL_MATCH, Color::Green, Color::Green, Color::Green)
                }
                CommitRefStatus::StagingPreprodMatch => (
                    STAGING_PREPROD_MATCH,
                    Color::Green,
                    Color::Green,
                    Color::Red,
                ),
                CommitRefStatus::PreprodProdMatch => (
                    PREPROD_PROD_MATCH,
                    PREPROD_PROD_MATCH,
                    Color::Green,
                    Color::Green,
                ),
                CommitRefStatus::CommitMissing => (
                    NONE_MATCH,
                    Color::DarkGray,
                    Color::DarkGray,
                    Color::DarkGray,
                ),
            };

            let is_active = selected_service_idx.is_none_or(|s| s == service_idx);

            let (staging_text, staging_color) = commit_cell(&service.staging, staging_ok);
            let (preprod_text, preprod_color) = commit_cell(&service.preproduction, preprod_ok);
            let (prod_text, prod_color) = commit_cell(&service.production, prod_ok);

            Row::new([
                Cell::from(Line::from(vec![
                    Span::styled("▍ ", Style::default().bg(service_color)),
                    Span::raw(" "),
                    Span::styled(config[service_idx].name.clone(), Style::default()),
                ])),
                Cell::from(staging_text).style(Style::default().fg(staging_color)),
                Cell::from(preprod_text).style(Style::default().fg(preprod_color)),
                Cell::from(prod_text).style(Style::default().fg(prod_color)),
            ])
            .style(row_style(is_active))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(23),
            Constraint::Percentage(23),
            Constraint::Percentage(24),
        ],
    )
    .block(Block::default())
    .header(headers);

    frame.render_stateful_widget(table, table_area, &mut state.table_state);

    // ── Render errors
    if let Some(service_idx) = state.table_state.selected() {
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
    
    let key = key_style();
    let desc = key_desc_style();
    let mut service_action_text = vec![
        Span::styled("[s]", key),
        Span::styled(" to scan the services  ", desc),
    ];

    if let Some(service) = state
        .table_state
        .selected()
        .and_then(|idx| state.services.get(idx))
    {
        match service.commit_ref_status() {
            CommitRefStatus::StagingPreprodMatch | CommitRefStatus::NothingMatches => {
                service_action_text.push(Span::styled("[o]", key));
                service_action_text.push(Span::styled(" to Open in browser  ", desc));
                service_action_text.push(Span::styled("[c]", key));
                service_action_text.push(Span::styled(" to Copy the url", desc));
            }
            _ => {}
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(service_action_text)).wrap(Wrap { trim: false }),
        action_area,
    );

    let legend_text = Line::from(vec![
        Span::styled("▍ ", Style::default().bg(ALL_MATCH)),
        Span::raw(" Up to date  "),
        Span::styled("▍ ", Style::default().bg(PREPROD_PROD_MATCH)),
        Span::raw(" New version in deployment pipeline  "),
        Span::styled("▍ ", Style::default().bg(STAGING_PREPROD_MATCH)),
        Span::raw(" Pending production deployment  "),
        Span::styled("▍ ", Style::default().bg(NONE_MATCH)),
        Span::raw(" Requires maintenance  "),
    ]);

    frame.render_widget(
        Paragraph::new(legend_text).wrap(Wrap { trim: false }),
        legend_area,
    );
}
