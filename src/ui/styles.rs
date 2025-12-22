// ui/styles.rs
use ratatui::style::{Style, Color};
use crate::state::BlockState;

pub fn block_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    }
}

pub fn is_menu_active(block: BlockState) -> bool {
    matches!(block, BlockState::Menu)
}

pub fn is_content_active(block: BlockState) -> bool {
    matches!(block, BlockState::Content)
}
