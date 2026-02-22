// ui/layout.rs
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Areas {
    pub menu: Rect,
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
        .constraints([Constraint::Min(30), Constraint::Percentage(90)])
        .split(outer_chunks[0]);

    Areas {
        menu: chunks[0],
        content: chunks[1],
        footer: outer_chunks[1],
    }
}
