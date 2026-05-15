use crate::state::app::{AppFocus, AppState};
use crate::tools::plugin::Plugin;
use crate::ui::styles::{key_desc_style, key_style};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, plugins: &[Box<dyn Plugin>]) {
    let (line1, line2) = build_lines(state, plugins);
    let footer = Paragraph::new(vec![line1, line2]).block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, area);
}

/// Named hint components. Each variant knows its own key label and description text.
/// Add a new variant here and it is available everywhere — change it here and it
/// updates everywhere.
enum Hint {
    // Navigation
    Navigate,
    // Global
    Quit,
    // Panel switching
    Tools,
    Config,
    Logs,
    // Actions
    Toggle,
    Add,
    Edit,
    // Form
    Save,
    NextField,
    NavigateFields,
    Cancel,
    Submit,
    // Popup
    Dismiss,
    // One-off with custom text (key_text, desc_text)
    Status(&'static str),
}

impl Hint {
    fn spans(&self) -> Vec<Span<'static>> {
        let k = key_style();
        let d = key_desc_style();
        match self {
            Hint::Navigate => vec![Span::styled("[↑↓←→]", k), Span::styled(" Navigate  ", d)],
            Hint::Quit => vec![Span::styled("[q]", k), Span::styled(" Quit", d)],
            Hint::Tools => vec![Span::styled("[1]", k), Span::styled(" Tools  ", d)],
            Hint::Config => vec![Span::styled("[2]", k), Span::styled(" Config  ", d)],
            Hint::Logs => vec![Span::styled("[3]", k), Span::styled(" Logs  ", d)],
            Hint::Toggle => vec![Span::styled("[return]", k), Span::styled(" Toggle  ", d)],
            Hint::Add => vec![Span::styled("[a]", k), Span::styled(" Add  ", d)],
            Hint::Edit => vec![Span::styled("[e]", k), Span::styled(" Edit  ", d)],
            Hint::Save => vec![Span::styled("[return]", k), Span::styled(" Save  ", d)],
            Hint::NextField => vec![Span::styled("[tab]", k), Span::styled(" Next field  ", d)],
            Hint::NavigateFields => vec![
                Span::styled("[↑↓]", k),
                Span::styled(" Navigate fields  ", d),
            ],
            Hint::Cancel => vec![Span::styled("[esc]", k), Span::styled(" Cancel  ", d)],
            Hint::Submit => vec![
                Span::styled("[return]", k),
                Span::styled(" Add ticket  ", d),
            ],
            Hint::Dismiss => vec![Span::styled("[any]", k), Span::styled(" Dismiss  ", d)],
            Hint::Status(text) => vec![Span::styled(*text, key_desc_style())],
        }
    }
}

fn hints(items: &[Hint]) -> Line<'static> {
    Line::from(items.iter().flat_map(|h| h.spans()).collect::<Vec<_>>())
}

fn build_lines(state: &AppState, plugins: &[Box<dyn Plugin>]) -> (Line<'static>, Line<'static>) {
    if state.has_popup() {
        return (hints(&[Hint::Dismiss]), Line::from(""));
    }

    match state.focus {
        AppFocus::JiraInput => (hints(&[Hint::Submit, Hint::Cancel]), Line::from("")),
        AppFocus::List => (
            hints(&[Hint::Navigate, Hint::Config, Hint::Logs, Hint::Quit]),
            Line::from(""),
        ),
        AppFocus::Tool => {
            if let Some(plugin) = plugins.iter().find(|p| p.id() == state.current_tool) {
                plugin.tool_hints()
            } else {
                (Line::from(""), Line::from(""))
            }
        }
        AppFocus::Config => (
            hints(&[
                Hint::Navigate,
                Hint::Toggle,
                Hint::Edit,
                Hint::Tools,
                Hint::Logs,
                Hint::Quit,
            ]),
            Line::from(""),
        ),
        AppFocus::Logs => (
            hints(&[Hint::Navigate, Hint::Tools, Hint::Config, Hint::Quit]),
            Line::from(""),
        ),
        AppFocus::ToolConfig(tool) => {
            if let Some(plugin) = plugins.iter().find(|p| p.id() == tool) {
                plugin.config_hints()
            } else {
                (Line::from(""), Line::from(""))
            }
        }
    }
}
