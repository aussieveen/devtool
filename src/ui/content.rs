// ui/content.rs
use ratatui::{
    Frame,
    widgets::{Paragraph, Block, Borders},
};
use crate::ui::styles;
use crate::state::State;

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &State,
) {
    let style = styles::block_style(
        styles::is_content_active(state.block),
    );

    let paragraph = Paragraph::new(state.content.content())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(state.content.title()),
        );

    frame.render_widget(paragraph, area);
}
