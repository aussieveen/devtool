use crate::{state::app::AppState, ui::styles};
// ui/tools
use ratatui::{
    Frame,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState) {
    let style = styles::block_style(styles::list_has_focus(state.effective_focus()));
    let shortcut = styles::panel_shortcut_style();

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled("[1]", shortcut),
        Span::raw(" Tools "),
    ]);

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
            .title(title),
    );

    frame.render_stateful_widget(menu, area, &mut state.tool_list.list_state);
}
