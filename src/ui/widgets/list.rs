use crate::{state::app::AppState, ui::styles};
// ui/tools
use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState) {
    let style = styles::block_style(styles::is_menu_active(state.focus));

    let menu = List::new(state.tool_list.items.iter().map(|i| ListItem::new(*i)))
        .block(Block::default().borders(Borders::ALL).title(" Tools "))
        .highlight_style(ratatui::style::Style::default().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true)
        .style(style);

    frame.render_stateful_widget(menu, area, &mut state.tool_list.list_state);
}
