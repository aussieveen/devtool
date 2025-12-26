use crate::{
    ui::styles,
    state::app_state::AppState
};
use ratatui::style::Stylize;
// ui/tools
use ratatui::{
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &mut AppState,
) {
    let style = styles::block_style(
        styles::is_menu_active(state.focus),
    );

    let menu = List::new(
        state.list.items.iter().map(|i| ListItem::new(*i))
    )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(" Tools ")
        )
        .highlight_style(ratatui::style::Style::default().reversed())
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(
        menu,
        area,
        &mut state.list.list_state,
    );
}
