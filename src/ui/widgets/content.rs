// ui/content.rs
use ratatui::{
    Frame,
    widgets::{Paragraph, Block, Borders},
};
use crate::state::state::State;
use crate::ui::styles;

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &State,
) {
    let content_block_border_style = styles::block_style(
        styles::is_content_active(state.block),
    );

    let paragraph = Paragraph::new(state.content.content())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(content_block_border_style)
                .title(format!(" {} ", state.content.title())),
        );

    frame.render_widget(paragraph, area);
}
