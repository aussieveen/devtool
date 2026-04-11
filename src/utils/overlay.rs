use ratatui::layout::{Constraint, Layout, Rect};

/// Returns a centred overlay area with the given percentage width and fixed row height.
pub fn overlay_area(area: Rect, percent_x: u16, vertical_length: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(vertical_length),
        Constraint::Fill(1),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(percent_x),
        Constraint::Fill(1),
    ])
    .split(vertical[1])[1]
}
