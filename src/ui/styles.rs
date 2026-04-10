use crate::state::app::AppFocus;
use ratatui::style::{Color, Modifier, Style};

// ── Theme palette ──────────────────────────────────────────────────────────
const ACTIVE_BORDER: Color = Color::Green;
const INACTIVE_BORDER: Color = Color::White;
pub const SELECTION_BG: Color = Color::Blue;
pub const SELECTION_FG: Color = Color::White;

pub fn block_style(active: bool) -> Style {
    if active {
        Style::default()
            .fg(ACTIVE_BORDER)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(INACTIVE_BORDER)
    }
}

/// Style to apply to a `List` or `Table` widget's `highlight_style`.
/// This replaces the old "dim everything else" approach with a blue background
/// on the selected row, matching lazydocker's default `selectedLineBgColor`.
pub fn selection_highlight() -> Style {
    Style::default()
        .bg(SELECTION_BG)
        .fg(SELECTION_FG)
        .add_modifier(Modifier::BOLD)
}

pub fn list_has_focus(focus: AppFocus) -> bool {
    matches!(focus, AppFocus::List)
}

pub fn tool_has_focus(focus: AppFocus) -> bool {
    matches!(focus, AppFocus::Tool | AppFocus::ToolConfig(_))
}

pub fn key_style() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn key_desc_style() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Style for `[1]`/`[2]` panel shortcut labels in sidebar titles.
pub fn panel_shortcut_style() -> Style {
    Style::default()
        .fg(ACTIVE_BORDER)
        .add_modifier(Modifier::BOLD)
}

/// Consistent border style for all active inline edit forms.
/// Uses the active border colour — when an inline edit form is active, its parent panel
/// loses focus (White border), making the edit form the sole Green element.
pub fn edit_border_style() -> Style {
    Style::default().fg(ACTIVE_BORDER)
}

#[cfg(test)]
mod tests {
    use crate::app::AppFocus;
    use crate::ui::styles::{block_style, list_has_focus, tool_has_focus};
    use ratatui::style::{Color, Modifier};
    use test_case::test_case;

    #[test]
    fn block_style_returns_inactive_style() {
        let actual = block_style(false);
        assert_eq!(actual.fg.unwrap(), Color::White);
    }

    #[test]
    fn block_style_returns_active_style() {
        let actual = block_style(true);
        assert_eq!(actual.fg.unwrap(), Color::Green);
        assert!(actual.add_modifier.contains(Modifier::BOLD));
    }

    #[test_case(AppFocus::List, true)]
    #[test_case(AppFocus::Tool, false)]
    fn list_has_focus_returns_expected(focus: AppFocus, expected: bool) {
        assert_eq!(list_has_focus(focus), expected)
    }

    #[test_case(AppFocus::List, false)]
    #[test_case(AppFocus::Tool, true)]
    #[test_case(AppFocus::ToolConfig(crate::app::Tool::ServiceStatus), true)]
    fn tool_has_focus_returns_expected(focus: AppFocus, expected: bool) {
        assert_eq!(tool_has_focus(focus), expected)
    }
}
