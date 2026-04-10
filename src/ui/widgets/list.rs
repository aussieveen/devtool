use crate::{state::app::AppState, ui::styles};
// ui/tools
use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState) {
    let style = styles::block_style(styles::list_has_focus(state.effective_focus()));

    let menu = List::new(
        state
            .tool_list
            .items
            .iter()
            .map(|i| ListItem::new(i.menu_entry())),
    )
    .highlight_style(styles::selection_highlight())
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title(" [1] Tools "),
    );

    frame.render_stateful_widget(menu, area, &mut state.tool_list.list_state);
}
