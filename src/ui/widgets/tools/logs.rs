use crate::state::log::{LogLevel, LogsItem};
use crate::ui::styles::block_style;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Padding};

pub fn render(frame: &mut Frame, area: Rect, log: &crate::state::log::LogState, focused: bool) {
    let border = block_style(focused);

    match log.selected_item {
        LogsItem::Activity => render_activity(frame, area, log, border),
        LogsItem::AppLog => render_app_log(frame, area, log, border),
    }
}

// 2 border + 2 block padding (Padding::horizontal(1) = 1 each side)
fn inner_width(area: Rect) -> usize {
    area.width.saturating_sub(4) as usize
}

/// Wrap a message string into lines of at most `width` chars.
/// Returns at least one element (may be an empty string).
fn wrap_message(msg: &str, width: usize) -> Vec<String> {
    if width == 0 || msg.is_empty() {
        return vec![msg.to_string()];
    }
    let mut lines = Vec::new();
    let mut remaining = msg;
    while !remaining.is_empty() {
        if remaining.len() <= width {
            lines.push(remaining.to_string());
            break;
        }
        // Try to break at a space boundary within the width.
        let break_at = remaining[..width]
            .rfind(' ')
            .map(|i| i + 1) // include the space on the current line, trim on next
            .unwrap_or(width);
        lines.push(remaining[..break_at].trim_end().to_string());
        remaining = remaining[break_at..].trim_start();
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn render_activity(
    frame: &mut Frame,
    area: Rect,
    log: &crate::state::log::LogState,
    border: Style,
) {
    let dim = Style::default().add_modifier(Modifier::DIM);
    let entries = log.visible_activity();
    let avail = inner_width(area);

    // prefix: " HH:MM  " (8) + source padded to 20 + " " (21) = 29 chars
    const PREFIX_LEN: usize = 29;
    let msg_width = avail.saturating_sub(PREFIX_LEN);

    let items: Vec<ListItem> = if entries.is_empty() {
        vec![ListItem::new(Line::styled("No activity yet.", dim))]
    } else {
        entries
            .iter()
            .map(|e| {
                let ts = e.timestamp.format("%H:%M").to_string();
                let source = format!("{:<20}", e.source);
                let chunks = wrap_message(&e.message, msg_width);

                let mut lines: Vec<Line> = Vec::with_capacity(chunks.len());
                // First line carries the full prefix
                lines.push(Line::from(vec![
                    Span::styled(format!(" {}  ", ts), dim),
                    Span::raw(format!("{} ", source)),
                    Span::raw(chunks[0].clone()),
                ]));
                // Continuation lines are indented to align with message start
                let indent = " ".repeat(PREFIX_LEN);
                for chunk in &chunks[1..] {
                    lines.push(Line::from(vec![
                        Span::raw(indent.clone()),
                        Span::raw(chunk.clone()),
                    ]));
                }
                ListItem::new(Text::from(lines))
            })
            .collect()
    };

    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border)
            .title(" Activity ")
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(widget, area);
}

fn render_app_log(frame: &mut Frame, area: Rect, log: &crate::state::log::LogState, border: Style) {
    let dim = Style::default().add_modifier(Modifier::DIM);
    let entries = log.visible_log();
    let avail = inner_width(area);

    // prefix: " HH:MM:SS  " (11) + "[LEVEL]  " (up to 9) + source padded to 12 + "  " (14) = ~34 chars
    // Use the widest variant (NOTICE = 6 chars label) for consistent wrapping
    const PREFIX_LEN: usize = 34;
    let msg_width = avail.saturating_sub(PREFIX_LEN);

    let items: Vec<ListItem> = if entries.is_empty() {
        vec![ListItem::new(Line::styled("No log entries yet.", dim))]
    } else {
        entries
            .iter()
            .map(|e| {
                let ts = e.timestamp.format("%H:%M:%S").to_string();
                let level_style = level_style(e.level);
                let source = format!("{:<12}", e.source);
                let chunks = wrap_message(&e.message, msg_width);

                let mut lines: Vec<Line> = Vec::with_capacity(chunks.len());
                lines.push(Line::from(vec![
                    Span::styled(format!(" {}  ", ts), dim),
                    Span::styled(format!("[{}]  ", e.level.label().trim()), level_style),
                    Span::styled(format!("{}  ", source), dim),
                    Span::raw(chunks[0].clone()),
                ]));
                let indent = " ".repeat(PREFIX_LEN);
                for chunk in &chunks[1..] {
                    lines.push(Line::from(vec![
                        Span::raw(indent.clone()),
                        Span::raw(chunk.clone()),
                    ]));
                }
                ListItem::new(Text::from(lines))
            })
            .collect()
    };

    let widget = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border)
            .title(" App Log ")
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(widget, area);
}

fn level_style(level: LogLevel) -> Style {
    match level {
        LogLevel::Emergency | LogLevel::Alert | LogLevel::Critical | LogLevel::Error => {
            Style::default().fg(Color::Red)
        }
        LogLevel::Warning => Style::default().fg(Color::Yellow),
        LogLevel::Notice => Style::default().fg(Color::Cyan),
        LogLevel::Info => Style::default(),
        LogLevel::Debug => Style::default().add_modifier(Modifier::DIM),
    }
}

#[cfg(test)]
mod tests {
    use super::wrap_message;

    #[test]
    fn wrap_message_short_fits_on_one_line() {
        let result = wrap_message("hello world", 40);
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn wrap_message_breaks_at_space_boundary() {
        let result = wrap_message(
            "Token request failed: service/staging — connection timed out",
            30,
        );
        assert!(result.len() > 1);
        for line in &result {
            assert!(line.len() <= 30, "line too long: {:?}", line);
        }
    }

    #[test]
    fn wrap_message_handles_no_spaces() {
        let result = wrap_message("averylongwordwithoutspaces", 10);
        assert_eq!(result, vec!["averylongw", "ordwithout", "spaces"]);
    }

    #[test]
    fn wrap_message_empty_string() {
        let result = wrap_message("", 20);
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn wrap_message_zero_width_returns_original() {
        let result = wrap_message("hello", 0);
        assert_eq!(result, vec!["hello"]);
    }
}
