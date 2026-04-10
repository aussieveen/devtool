use crate::{state::app::AppState, ui::styles};
// ui/tools
use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState) {
    let style = styles::block_style(styles::list_has_focus(state.effective_focus()));
    let selected = state.tool_list.list_state.selected();

    let menu = List::new(
        state
            .tool_list
            .items
            .iter()
            .enumerate()
            .map(|(idx, i)| {
                ListItem::new(i.menu_entry())
                    .style(styles::list_style(selected.is_none_or(|s| s == idx)))
            }),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title(" Tools "),
    );

    frame.render_stateful_widget(menu, area, &mut state.tool_list.list_state);
}
