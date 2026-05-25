use super::state::{Commit, CommitRefStatus, ServiceStatus};
use crate::config::model::ServiceStatusConfig;
use crate::ui::styles::selection_highlight;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &mut ServiceStatus,
    config: &[ServiceStatusConfig],
) {
    if config.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(
                "No services configured — press [2] then Enter on Service Status to configure.",
            ))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    const ALL_MATCH: Color = Color::Green;
    const NONE_MATCH: Color = Color::Red;
    const PREPROD_PROD_MATCH: Color = Color::Cyan;
    const STAGING_PREPROD_MATCH: Color = Color::Yellow;

    let commit_cell = |commit: &Commit, ok_color: Color| -> (String, Color) {
        match commit {
            Commit::Fetching => ("…".to_string(), Color::DarkGray),
            Commit::Empty => ("—".to_string(), Color::DarkGray),
            Commit::Error(_) => ("Error".to_string(), NONE_MATCH),
            Commit::Ok(_) => (commit.short_value().unwrap_or_default(), ok_color),
        }
    };

    let table_length = (state.services.len() + 1) as u16; // services + header row

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(table_length), // table
            Constraint::Min(0),               // filler
            Constraint::Length(2),            // color legend
        ])
        .split(area);

    let table_area = vertical[0];
    let legend_area = vertical[2];

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
    .row_highlight_style(selection_highlight())
    .block(Block::default())
    .header(headers);

    frame.render_stateful_widget(table, table_area, &mut state.table_state);

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
