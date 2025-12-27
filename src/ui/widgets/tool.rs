// ui/tool
use ratatui::{
    Frame,
    widgets::{Block, Borders},
};
use crate::state::app_state::AppState;
use crate::ui::styles;

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &mut AppState,
) {
    let content_block_border_style = styles::block_style(
        styles::is_content_active(state.focus),
    );

    let pane = Block::default()
        .borders(Borders::ALL)
        .border_style(content_block_border_style)
        .title(format!(" {} ", state.current_tool.title()));

    let inner = pane.inner(area);

    frame.render_widget(pane,area);

    state.current_tool.render(frame, inner, &mut state.diffchecker, &mut state.tokengenerator);
}
