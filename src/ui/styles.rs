// ui/styles.rs
use ratatui::style::{Style, Color};
use crate::state::focus::AppFocus;

pub fn block_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    }
}

pub fn is_menu_active(focus: AppFocus) -> bool {
    matches!(focus, AppFocus::List)
}

pub fn is_content_active(focus: AppFocus) -> bool {
    matches!(focus, AppFocus::Tool)
}

pub fn list_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}