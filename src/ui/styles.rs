use crate::state::app::AppFocus;
use ratatui::style::{Color, Modifier, Style};

pub fn block_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

pub fn list_has_focus(focus: AppFocus) -> bool {
    matches!(focus, AppFocus::List)
}

pub fn tool_has_focus(focus: AppFocus) -> bool {
    matches!(focus, AppFocus::Tool)
}

pub fn list_style(active: bool) -> Style {
    if active {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
pub fn key_style() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn key_desc_style() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

pub fn row_style(active: bool) -> Style {
    if active {
        Style::default()
    } else {
        Style::default().add_modifier(Modifier::DIM)
    }
}

#[cfg(test)]
mod tests {
    use crate::app::AppFocus;
    use crate::ui::styles::{block_style, list_has_focus, list_style, tool_has_focus};
    use ratatui::style::Color;
    use test_case::test_case;

    #[test]
    fn block_style_returns_inactive_style() {
        let actual = block_style(false);
        assert_eq!(actual.fg.unwrap(), Color::DarkGray);
    }

    #[test]
    fn block_style_returns_active_style() {
        let actual = block_style(true);
        assert_eq!(actual.fg.unwrap(), Color::Cyan);
    }

    #[test_case(AppFocus::List, true)]
    #[test_case(AppFocus::Tool, false)]
    fn list_has_focus_returns_expected(focus: AppFocus, expected: bool) {
        assert_eq!(list_has_focus(focus), expected)
    }

    #[test_case(AppFocus::List, false)]
    #[test_case(AppFocus::Tool, true)]
    fn tool_has_focus_returns_expected(focus: AppFocus, expected: bool) {
        assert_eq!(tool_has_focus(focus), expected)
    }

    #[test]
    fn list_style_returns_inactive_style() {
        let actual = list_style(false);
        assert_eq!(actual.fg.unwrap(), Color::DarkGray);
    }

    #[test]
    fn list_style_returns_active_style() {
        let actual = list_style(true);
        assert!(actual.fg.is_none());
    }
}
