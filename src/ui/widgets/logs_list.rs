use crate::state::app::{AppFocus, AppState};
use crate::state::log::LogsItem;
use crate::ui::styles::{block_style, panel_shortcut_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let is_focused = matches!(state.effective_focus(), AppFocus::Logs);
    let border_style = block_style(is_focused);
    let shortcut = panel_shortcut_style();
    let item_style = if is_focused {
        Style::default()
    } else {
        Style::default().add_modifier(Modifier::DIM)
    };

    let highlight = if is_focused { selection_highlight() } else { Style::default() };

    let unread_dot = if state.log.has_unread_activity() {
        "● "
    } else {
        "  "
    };

    let items = vec![
        ListItem::new(format!("{}{}", unread_dot, "Activity")).style(item_style),
        ListItem::new("  App Log").style(item_style),
    ];

    let selected_idx = match state.log.selected_item {
        LogsItem::Activity => 0,
        LogsItem::AppLog => 1,
    };
    let mut list_state = ListState::default().with_selected(Some(selected_idx));
    if is_focused {
        *list_state.offset_mut() = 0;
    }

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled("[3]", shortcut),
        Span::raw(" Logs "),
    ]);

    let list = List::new(items)
        .highlight_style(highlight)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        );

    frame.render_stateful_widget(list, area, &mut list_state);
}
