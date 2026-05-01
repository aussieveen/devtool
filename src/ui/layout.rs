// ui/layout.rs
use crate::state::app::AppFocus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Areas {
    pub tools_list: Rect,
    pub config_list: Rect,
    pub logs_list: Rect,
    pub content: Rect,
    pub footer: Rect,
}

pub fn main(area: Rect, focus: AppFocus, tool_count: usize) -> Areas {
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(outer_chunks[0]);

    // Collapsed height: just border + title line (3 rows).
    // Active panel gets Min(0) to fill remaining space.
    // Tools panel when inactive: border + title + one row per tool + padding = tool_count + 2,
    // capped so it doesn't grow unboundedly when another panel is active.
    const COLLAPSED: u16 = 3;
    let tools_inactive_height = (tool_count as u16 + 2).max(COLLAPSED);

    let (tools_c, config_c, logs_c) = match focus {
        AppFocus::List => (
            Constraint::Min(0),
            Constraint::Length(COLLAPSED),
            Constraint::Length(COLLAPSED),
        ),
        AppFocus::Config | AppFocus::ToolConfig(_) => (
            Constraint::Length(tools_inactive_height),
            Constraint::Min(0),
            Constraint::Length(COLLAPSED),
        ),
        AppFocus::Logs => (
            Constraint::Length(tools_inactive_height),
            Constraint::Length(COLLAPSED),
            Constraint::Min(0),
        ),
        // Tool / JiraInput / popup: tools list stays expanded (user is inside a tool)
        AppFocus::Tool | AppFocus::JiraInput => (
            Constraint::Length(tools_inactive_height),
            Constraint::Length(COLLAPSED),
            Constraint::Length(COLLAPSED),
        ),
    };

    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([tools_c, config_c, logs_c])
        .split(chunks[0]);

    Areas {
        tools_list: sidebar[0],
        config_list: sidebar[1],
        logs_list: sidebar[2],
        content: chunks[1],
        footer: outer_chunks[1],
    }
}
