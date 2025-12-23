use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use crate::state::app_state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState){
    let test: String = state.diff_checker.services.iter().map(|s| s.config.name.clone()).collect();
    frame.render_widget(Paragraph::new(test), area)
}