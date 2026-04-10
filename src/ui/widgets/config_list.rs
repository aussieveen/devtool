use crate::state::app::{AppFocus, AppState};
use crate::ui::styles::{block_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let is_focused = matches!(state.effective_focus(), AppFocus::Config);
    let border_style = block_style(is_focused);

    let items = state.config_editor.items.iter().map(|item| {
        let checkbox = if item.enabled { "[✓]" } else { "[ ]" };
        let text = format!("{} {}", checkbox, item.tool.menu_entry());
        ListItem::new(text)
    });

    let list = List::new(items)
        .highlight_style(selection_highlight())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" [2] Config "),
        );

    frame.render_stateful_widget(list, area, &mut state.config_editor.list_state);
}
