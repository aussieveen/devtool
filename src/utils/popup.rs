use ratatui::layout::{Constraint, Flex, Layout, Rect};

pub fn popup_area(area: Rect, percent_x: u16, vertical_length: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Max(vertical_length)]).flex(Flex::SpaceAround);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}