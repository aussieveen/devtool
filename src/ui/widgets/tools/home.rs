use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
pub fn render(frame: &mut Frame, area: Rect) {
    frame.render_widget(Paragraph::new("HOME"), area)
}