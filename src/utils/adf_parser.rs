use ratatui::prelude::Style;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use crate::client::jira::models::{Content, Description};

pub fn parse(description: Description, limit: Option<usize>) -> Vec<Line<'static>>
{
    let mut parsed = vec![];
    for c in description.content{
        parsed = [parsed, parse_content(c)].concat();
        if let Some(l) = limit && parsed.len() > l{
            parsed.truncate(l);
            return parsed;
        }
    }
    parsed
}

fn parse_content(content: Content) -> Vec<Line<'static>>
{
    let mut parsed = vec![];
    match content.r#type.as_str() {
        "paragraph" => {
            if let Some(paragraph_content) = content.content &&
                let Some(first_content) = paragraph_content.first(){
                return parse_content(first_content.clone())
            }
            parsed
        },
        "text" => {
            if let Some(text) = content.text{
                let mut style = Style::default();
                // marks
                if let Some(marks) = content.marks{
                    for m in marks{
                        let modifier: Option<Modifier> = match m.r#type.as_str() {
                            "strong" => Some(Modifier::BOLD),
                            _ => None
                        };
                        if let Some(m) = modifier {
                            style = style.add_modifier(m);
                        }
                    }
                }
                parsed.push(Line::from(Span::styled(text, style)));
            }
            parsed
        }
        _ => parsed
    }
}