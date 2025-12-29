// ui/styles.rs
use crate::state::app::AppFocus;
use ratatui::style::{Color, Style};

pub fn block_style(active: bool) -> Style {
    if active {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
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
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
