// ui/layout.rs
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Areas {
    pub tools_list: Rect,
    pub config_list: Rect,
    pub content: Rect,
    pub footer: Rect,
}

pub fn main(area: Rect) -> Areas {
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(outer_chunks[0]);

    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    Areas {
        tools_list: sidebar[0],
        config_list: sidebar[1],
        content: chunks[1],
        footer: outer_chunks[1],
    }
}
