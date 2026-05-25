use crate::state::app::{AppFocus, AppState};
use crate::tools::plugin::{Plugin, menu_entry_for};
use crate::ui::styles::{block_style, panel_shortcut_style, selection_highlight};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState, plugins: &[Box<dyn Plugin>]) {
    let is_focused = matches!(
        state.effective_focus(),
        AppFocus::Config | AppFocus::ToolConfig(_)
    );
    let border_style = block_style(is_focused);
    let shortcut = panel_shortcut_style();
    let item_style = if is_focused {
        Style::default()
    } else {
        Style::default().add_modifier(Modifier::DIM)
    };

    let highlight = if is_focused {
        selection_highlight()
    } else {
        Style::default()
    };

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled("[2]", shortcut),
        Span::raw(" Config "),
    ]);

    let items = state.config_editor.items.iter().map(|item| {
        let checkbox = if item.enabled { "[✓]" } else { "[ ]" };
        let text = format!("{} {}", checkbox, menu_entry_for(plugins, item.tool));
        ListItem::new(text).style(item_style)
    });

    let list = List::new(items).highlight_style(highlight).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title),
    );

    if is_focused {
        *state.config_editor.list_state.offset_mut() = 0;
    }

    frame.render_stateful_widget(list, area, &mut state.config_editor.list_state);
}
