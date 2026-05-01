use crate::state::app::{AppFocus, AppState, Tool};
use crate::state::token_generator::Token;
use crate::ui::styles::{key_desc_style, key_style};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (line1, line2) = build_lines(state);
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
    Remove,
    Scan,
    Generate,
    OpenInBrowser,
    CopyUrl,
    CopyToken,
    Retry,
    MoveItem,
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
            Hint::Navigate           => vec![Span::styled("[↑↓←→]", k),        Span::styled(" Navigate  ", d)],
            Hint::Quit               => vec![Span::styled("[q]", k),      Span::styled(" Quit", d)],
            Hint::Tools              => vec![Span::styled("[1]", k),          Span::styled(" Tools  ", d)],
            Hint::Config             => vec![Span::styled("[2]", k),          Span::styled(" Config  ", d)],
            Hint::Logs               => vec![Span::styled("[3]", k),          Span::styled(" Logs  ", d)],
            Hint::Toggle             => vec![Span::styled("[return]", k),      Span::styled(" Toggle  ", d)],
            Hint::Add                => vec![Span::styled("[a]", k),          Span::styled(" Add  ", d)],
            Hint::Edit               => vec![Span::styled("[e]", k),          Span::styled(" Edit  ", d)],
            Hint::Remove             => vec![Span::styled("[x]", k),          Span::styled(" Remove  ", d)],
            Hint::Scan               => vec![Span::styled("[s]", k),          Span::styled(" Scan  ", d)],
            Hint::Generate           => vec![Span::styled("[return]", k),     Span::styled(" Generate  ", d)],
            Hint::OpenInBrowser      => vec![Span::styled("[o]", k),          Span::styled(" Open in browser  ", d)],
            Hint::CopyUrl            => vec![Span::styled("[c]", k),          Span::styled(" Copy url  ", d)],
            Hint::CopyToken          => vec![Span::styled("[c]", k),          Span::styled(" Copy token  ", d)],
            Hint::Retry              => vec![Span::styled("[return]", k),     Span::styled(" Retry  ", d)],
            Hint::MoveItem           => vec![Span::styled("[shift+↑↓]", k), Span::styled(" Move  ", d)],
            Hint::Save               => vec![Span::styled("[return]", k),      Span::styled(" Save  ", d)],
            Hint::NextField          => vec![Span::styled("[tab]", k),        Span::styled(" Next field  ", d)],
            Hint::NavigateFields     => vec![Span::styled("[↑↓]", k),        Span::styled(" Navigate fields  ", d)],
            Hint::Cancel             => vec![Span::styled("[esc]", k),        Span::styled(" Cancel  ", d)],
            Hint::Submit => vec![Span::styled("[return]", k), Span::styled(" Add ticket  ", d)],
            Hint::Dismiss => vec![Span::styled("[any]", k), Span::styled(" Dismiss  ", d)],
            Hint::Status(text)       => vec![Span::styled(*text, key_desc_style())],
        }
    }
}

fn hints(items: &[Hint]) -> Line<'static> {
    Line::from(items.iter().flat_map(|h| h.spans()).collect::<Vec<_>>())
}

fn build_lines(state: &AppState) -> (Line<'static>, Line<'static>) {
    if state.has_popup() {
        return (hints(&[Hint::Dismiss]), Line::from(""));
    }

    match state.focus {
        AppFocus::JiraInput => (
            hints(&[Hint::Submit, Hint::Cancel]),
            Line::from(""),
        ),
        AppFocus::List => (
            hints(&[Hint::Navigate, Hint::Config, Hint::Logs, Hint::Quit]),
            Line::from(""),
        ),
        AppFocus::Tool => match state.current_tool {
            Tool::ServiceStatus => service_status_tool_hints(state),
            Tool::TokenGenerator => token_generator_tool_hints(state),
            Tool::Jira => jira_tool_hints(state),
        },
        AppFocus::Config => (
            hints(&[Hint::Navigate, Hint::Toggle, Hint::Edit, Hint::Tools, Hint::Logs, Hint::Quit]),
            Line::from(""),
        ),
        AppFocus::Logs => (
            hints(&[Hint::Navigate, Hint::Tools, Hint::Config, Hint::Quit]),
            Line::from(""),
        ),
        AppFocus::ToolConfig(Tool::ServiceStatus) => service_status_config_hints(state),
        AppFocus::ToolConfig(Tool::TokenGenerator) => token_generator_config_hints(state),
        AppFocus::ToolConfig(Tool::Jira) => jira_config_hints(state),
    }
}

fn service_status_tool_hints(state: &AppState) -> (Line<'static>, Line<'static>) {
    let line2 = if state.service_status.has_link() {
        hints(&[Hint::OpenInBrowser, Hint::CopyUrl])
    } else {
        Line::from("")
    };
    (
        hints(&[Hint::Navigate, Hint::Scan, Hint::Quit]),
        line2,
    )
}

fn token_generator_tool_hints(state: &AppState) -> (Line<'static>, Line<'static>) {
    let line2 = match state.token_generator.get_token_for_selected_service_env() {
        Token::Idle => Line::from(""),
        Token::Requesting => hints(&[Hint::Status("Generating token…")]),
        Token::Ready(_) => hints(&[Hint::CopyToken]),
        Token::Error => hints(&[Hint::Retry]),
    };
    (
        hints(&[Hint::Navigate, Hint::Generate, Hint::Quit]),
        line2,
    )
}

fn jira_tool_hints(state: &AppState) -> (Line<'static>, Line<'static>) {
    let line2 = if state.jira.list_state.selected().is_some() {
        hints(&[Hint::Remove, Hint::OpenInBrowser, Hint::MoveItem])
    } else {
        Line::from("")
    };
    (
        hints(&[Hint::Navigate, Hint::Add, Hint::Quit]),
        line2,
    )
}

fn service_status_config_hints(state: &AppState) -> (Line<'static>, Line<'static>) {
    if state.service_status_config_editor.has_open_form() {
        return edit_form_lines();
    }
    let line2 = if state.service_status_config_editor.table_state.selected().is_some() {
        hints(&[Hint::Edit, Hint::Remove])
    } else {
        Line::from("")
    };
    (
        hints(&[Hint::Navigate, Hint::Add, Hint::Quit]),
        line2,
    )
}

fn token_generator_config_hints(state: &AppState) -> (Line<'static>, Line<'static>) {
    use crate::state::token_generator_config::ConfigFocus;
    if state.token_generator_config_editor.has_open_form() {
        return edit_form_lines();
    }
    match state.token_generator_config_editor.config_focus {
        ConfigFocus::Auth0 => (
            hints(&[Hint::Add, Hint::Edit, Hint::Quit]),
            Line::from("")
        ),
        ConfigFocus::Services => {
            let line2 = if state.token_generator_config_editor.table_state.selected().is_some() {
                hints(&[Hint::Edit, Hint::Remove])
            } else {
                Line::from("")
            };
            (
                hints(&[Hint::Navigate, Hint::Add, Hint::Quit]),
                line2,
            )
        }
    }
}

fn jira_config_hints(state: &AppState) -> (Line<'static>, Line<'static>) {
    if state.jira_config_editor.has_open_form() {
        return edit_form_lines();
    }
    (
        hints(&[Hint::Edit, Hint::Quit]),
        Line::from(""),
    )
}

/// Shared footer content when any inline edit form is active.
fn edit_form_lines() -> (Line<'static>, Line<'static>) {
    (
        hints(&[Hint::Save, Hint::NextField, Hint::NavigateFields]),
        hints(&[Hint::Cancel]),
    )
}

