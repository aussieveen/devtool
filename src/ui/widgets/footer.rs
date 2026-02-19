use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect) {
    let key = Style::default().fg(Color::Cyan);
    let desc = Style::default().add_modifier(Modifier::DIM);

    let line1 = Line::from(vec![
        Span::styled("[← →]", key),
        Span::styled("  Switch panels/lists  ", desc),
        Span::styled("[↑ ↓]", key),
        Span::styled("  Move in list  ", desc),
        Span::styled("[d]", key),
        Span::styled("  Dismiss Popup  ", desc),
        Span::styled("[esc|q]", key),
        Span::styled("  Quit", desc),
    ]);
    let line2 = Line::styled("More actions are shown within each tool.", desc);

    let footer = Paragraph::new(vec![line1, line2]).block(
        Block::default()
            .borders(Borders::TOP)
            .title(" Help ")
            .title_alignment(Alignment::Center),
    );

    frame.render_widget(footer, area);
}
