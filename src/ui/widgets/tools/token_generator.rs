use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, area: Rect){
    let vertical_break = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Percentage(99),
        ])
        .split(area);

    let inner_horizonal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .split(vertical_break[0]);

    frame.render_widget(Paragraph::new("SERVICE"), inner_horizonal[0]);
    frame.render_widget(Paragraph::new("ENV"), inner_horizonal[1]);
    frame.render_widget(Paragraph::new("TOKEN MESSAGING"), vertical_break[1]);

}