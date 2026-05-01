use crate::{state::app::AppState, ui::styles};
// ui/tools
use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState) {
    let is_focused = styles::list_has_focus(state.effective_focus());
    let style = styles::block_style(is_focused);
    let shortcut = styles::panel_shortcut_style();
    let item_style = if is_focused {
        Style::default()
    } else {
        Style::default().add_modifier(Modifier::DIM)
    };

    let highlight = if is_focused { styles::selection_highlight() } else { Style::default() };

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
            .map(|i| ListItem::new(i.menu_entry()).style(item_style)),
    )
    .highlight_style(highlight)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title(title),
    );

    if is_focused {
        *state.tool_list.list_state.offset_mut() = 0;
    }

    frame.render_stateful_widget(menu, area, &mut state.tool_list.list_state);
}
