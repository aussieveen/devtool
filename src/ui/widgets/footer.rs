use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(
        "← →  Switch panels/lists  ↑ ↓  Move in list  esc|q Quit\n\
     More actions are shown within each tool.",
    )
    .block(
        Block::default()
            .borders(Borders::TOP)
            .title(" Help ")
            .title_alignment(Alignment::Center),
    )
    .style(Style::default().add_modifier(Modifier::DIM));

    frame.render_widget(footer, area);
}
