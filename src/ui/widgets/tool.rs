use crate::state::app::{AppState, Tool};
use crate::ui::styles;
use ratatui::prelude::Alignment;
use ratatui::{
    Frame,
    widgets::{Block, Borders},
};
use crate::config::Config;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState, config: &Config) {
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

    frame.render_widget(pane, area);

    state.current_tool.render(
        frame,
        inner,
        config,
        &mut state.service_status,
        &mut state.token_generator,
        &mut state.jira,
    );
}
