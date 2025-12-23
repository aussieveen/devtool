// ui/styles.rs
use ratatui::style::{Style, Color};
use crate::state::focus::Focus;

pub fn block_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    }
}

pub fn is_menu_active(focus: Focus) -> bool {
    matches!(focus, Focus::List)
}

pub fn is_content_active(focus: Focus) -> bool {
    matches!(focus, Focus::Tool)
}
