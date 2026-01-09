use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect) {
    let text_blocks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Diff title
            Constraint::Length(3), // Diff description
            Constraint::Length(1), // Auth title
            Constraint::Length(3), // Auth description
        ])
        .split(area);

    let title_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    frame.render_widget(
        Line::styled("Service Status", title_style).alignment(Alignment::Center),
        text_blocks[0],
    );

    frame.render_widget(
        Paragraph::new("Display the status of services. Using health checks, the tool compares the commit references between the services and generates a status. It will also produce a Github compare url should preproduction and production differ")
            .wrap(Wrap { trim: true }),
        text_blocks[1]
    );

    frame.render_widget(
        Line::styled("Auth0 Token Generator", title_style).alignment(Alignment::Center),
        text_blocks[2],
    );

    frame.render_widget(
        Paragraph::new("Generate a machine-to-machine (M2M) token. Select an API and environment to quickly produce an access token for service authentication.")
            .wrap(Wrap { trim: true }),
        text_blocks[3]
    );
}
