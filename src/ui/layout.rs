// ui/layout.rs
use ratatui::layout::{Layout, Direction, Constraint, Rect};

pub struct Areas {
    pub menu: Rect,
    pub content: Rect,
}

pub fn main(area: Rect) -> Areas {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),
            Constraint::Percentage(90),
        ])
        .split(area);

    Areas {
        menu: chunks[0],
        content: chunks[1],
    }
}
