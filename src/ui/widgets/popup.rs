use ratatui::Frame;
use ratatui::prelude::Line;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Clear, Paragraph};
use crate::popup::model::Popup;
use crate::utils::overlay::overlay_area;

pub enum Type{
    Error,
    Confirm,
    Success,
}

pub enum Part {
    Key(&'static str),
    Text(&'static str),
}

pub fn render(frame: &mut Frame, popup: &Popup)
{
    let color = match popup.popup_type {
        Type::Error => Color::Red,
        Type::Confirm => Color::Gray,
        Type::Success => Color::Green
    };

    let style = Style::default().fg(color).add_modifier(Modifier::BOLD);
    let dim = Style::default().add_modifier(Modifier::DIM);
    let key = crate::ui::styles::key_style();

    let block = Block::bordered().border_style(style).title_style(style);
    let body_line = Line::from(popup.parts.iter().map(|part| match part {
        Part::Key(s) => Span::styled(*s, key),
        Part::Text(s) => Span::styled(*s, dim),
    }).collect::<Vec<_>>());
    let content = Paragraph::new(vec![
        Line::from(Span::styled(popup.title.as_str(), style)),
        Line::from(""),
        body_line,
    ]).block(block);

    let area = overlay_area(frame.area(), 40, 5);
    frame.render_widget(Clear, area);
    frame.render_widget(content, area);
}