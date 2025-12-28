use color_eyre::owo_colors::OwoColorize;
// ui/tool
use ratatui::{
    Frame,
    widgets::{Block, Borders},
};
use ratatui::prelude::{Alignment, Color};
use ratatui::style::Style;
use crate::state::app_state::{AppState, Tool};
use crate::ui::styles;

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &mut AppState,
) {
    let content_block_border_style = styles::block_style(
        styles::is_content_active(state.focus) || matches!(state.current_tool, Tool::Home),
    );

    let pane = Block::default()
        .borders(Borders::ALL)
        .border_style(content_block_border_style)
        .title(format!(" {} ", state.current_tool.title()))
        .title_alignment(Alignment::Center)
        .style(content_block_border_style);

    let inner = pane.inner(area);

    frame.render_widget(pane,area);

    state.current_tool.render(frame, inner, &mut state.diffchecker, &mut state.tokengenerator);
}
