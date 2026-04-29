use crate::state::app::{AppFocus, AppState, Tool};
use crate::state::token_generator::Token;
use crate::ui::styles::{key_desc_style, key_style};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let key = key_style();
    let desc = key_desc_style();

    let (line1, line2) = build_lines(state, key, desc);

    let footer = Paragraph::new(vec![line1, line2]).block(Block::default().borders(Borders::TOP));

    frame.render_widget(footer, area);
}

fn build_lines<'a>(state: &AppState, key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    if state.error.is_some() {
        return (hints(&[("[d]", key, " Dismiss", desc)]), Line::from(""));
    }

    match state.focus {
        AppFocus::JiraInput => (
            hints(&[
                ("[enter]", key, " Add ticket  ", desc),
                ("[esc]", key, " Cancel", desc),
            ]),
            Line::from(""),
        ),
        AppFocus::List => (
            hints(&[
                ("[↑↓]", key, " Navigate  ", desc),
                ("[→]", key, " Open tool  ", desc),
                ("[2]", key, " Config  ", desc),
                ("[3]", key, " Logs  ", desc),
                ("[q/esc]", key, " Quit", desc),
            ]),
            Line::from(""),
        ),
        AppFocus::Tool => match state.current_tool {
            Tool::ServiceStatus => service_status_tool_hints(state, key, desc),
            Tool::TokenGenerator => token_generator_tool_hints(state, key, desc),
            Tool::Jira => jira_tool_hints(state, key, desc),
        },
        AppFocus::Config => (
            hints(&[
                ("[↑↓]", key, " Navigate  ", desc),
                ("[enter]", key, " Toggle  ", desc),
                ("[→]", key, " Edit config  ", desc),
                ("[1]", key, " Tools  ", desc),
                ("[q/esc]", key, " Quit", desc),
            ]),
            Line::from(""),
        ),
        AppFocus::Logs => (
            hints(&[
                ("[↑↓]", key, " Switch panel  ", desc),
                ("[1]", key, " Tools  ", desc),
                ("[2]", key, " Config  ", desc),
                ("[q/esc]", key, " Quit", desc),
            ]),
            Line::from(""),
        ),
        AppFocus::ToolConfig(Tool::ServiceStatus) => service_status_config_hints(state, key, desc),
        AppFocus::ToolConfig(Tool::TokenGenerator) => {
            token_generator_config_hints(state, key, desc)
        }
        AppFocus::ToolConfig(Tool::Jira) => jira_config_hints(state, key, desc),
    }
}

fn service_status_tool_hints<'a>(state: &AppState, key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    let line2 = if state.service_status.has_link() {
        hints(&[
            ("[o]", key, " Open in browser  ", desc),
            ("[c]", key, " Copy url", desc),
        ])
    } else {
        Line::from("")
    };
    (
        hints(&[
            ("[↑↓]", key, " Navigate  ", desc),
            ("[s]", key, " Scan  ", desc),
            ("[←]", key, " Tool list  ", desc),
            ("[2]", key, " Config  ", desc),
            ("[3]", key, " Logs  ", desc),
            ("[q/esc]", key, " Quit", desc),
        ]),
        line2,
    )
}

fn token_generator_tool_hints<'a>(state: &AppState, key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    let line2 = match state.token_generator.get_token_for_selected_service_env() {
        Token::Idle => Line::from(""),
        Token::Requesting => hints(&[("Generating token…", desc, "", desc)]),
        Token::Ready(_) => hints(&[("[c]", key, " Copy token", desc)]),
        Token::Error => hints(&[("[return]", key, " Retry", desc)]),
    };
    (
        hints(&[
            ("[←→]", key, " Switch panel  ", desc),
            ("[↑↓]", key, " Navigate  ", desc),
            ("[return]", key, " Generate  ", desc),
            ("[2]", key, " Config  ", desc),
            ("[q/esc]", key, " Quit", desc),
        ]),
        line2,
    )
}

fn jira_tool_hints<'a>(state: &AppState, key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    let line2 = if state.jira.list_state.selected().is_some() {
        hints(&[
            ("[x]", key, " Remove  ", desc),
            ("[o]", key, " Open in browser  ", desc),
            ("[shift+↑↓]", key, " Move", desc),
        ])
    } else {
        Line::from("")
    };
    (
        hints(&[
            ("[↑↓]", key, " Navigate  ", desc),
            ("[a]", key, " Add ticket  ", desc),
            ("[←]", key, " Tool list  ", desc),
            ("[2]", key, " Config  ", desc),
            ("[q/esc]", key, " Quit", desc),
        ]),
        line2,
    )
}

fn service_status_config_hints<'a>(state: &AppState, key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    if state.service_status_config_editor.has_open_form() {
        return edit_form_lines(key, desc);
    }
    let line2 = if state.service_status_config_editor.table_state.selected().is_some() {
        hints(&[("[e]", key, " Edit  ", desc), ("[x]", key, " Remove", desc)])
    } else {
        Line::from("")
    };
    (
        hints(&[
            ("[↑↓]", key, " Navigate  ", desc),
            ("[a]", key, " Add  ", desc),
            ("[←]", key, " Back  ", desc),
            ("[q/esc]", key, " Quit", desc),
        ]),
        line2,
    )
}

fn token_generator_config_hints<'a>(state: &AppState, key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    use crate::state::token_generator_config::ConfigFocus;
    if state.token_generator_config_editor.has_open_form() {
        return edit_form_lines(key, desc);
    }
    match state.token_generator_config_editor.config_focus {
        ConfigFocus::Auth0 => (
            hints(&[
                ("[tab]", key, " Switch section  ", desc),
                ("[a]", key, " Add service  ", desc),
                ("[←]", key, " Back  ", desc),
                ("[q/esc]", key, " Quit", desc),
            ]),
            hints(&[("[e]", key, " Edit", desc)]),
        ),
        ConfigFocus::Services => {
            let line2 = if state.token_generator_config_editor.table_state.selected().is_some() {
                hints(&[("[e]", key, " Edit  ", desc), ("[x]", key, " Remove", desc)])
            } else {
                Line::from("")
            };
            (
                hints(&[
                    ("[↑↓]", key, " Navigate  ", desc),
                    ("[tab]", key, " Switch section  ", desc),
                    ("[a]", key, " Add service  ", desc),
                    ("[←]", key, " Back  ", desc),
                    ("[q/esc]", key, " Quit", desc),
                ]),
                line2,
            )
        }
    }
}

fn jira_config_hints<'a>(state: &AppState, key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    if state.jira_config_editor.has_open_form() {
        return edit_form_lines(key, desc);
    }
    (
        hints(&[
            ("[e]", key, " Edit config  ", desc),
            ("[←]", key, " Back  ", desc),
            ("[q/esc]", key, " Quit", desc),
        ]),
        Line::from(""),
    )
}

/// Shared footer content when any inline edit form is active.
fn edit_form_lines<'a>(key: Style, desc: Style) -> (Line<'a>, Line<'a>) {
    (
        hints(&[
            ("[enter]", key, " Save  ", desc),
            ("[tab]", key, " Next field  ", desc),
            ("[↑↓]", key, " Navigate fields", desc),
        ]),
        hints(&[("[esc]", key, " Cancel", desc)]),
    )
}

/// Build a Line from a sequence of (key_text, key_style, desc_text, desc_style) tuples.
fn hints<'a>(items: &[(&'a str, Style, &'a str, Style)]) -> Line<'a> {
    let spans: Vec<Span<'a>> = items
        .iter()
        .flat_map(|(k, ks, d, ds)| [Span::styled(*k, *ks), Span::styled(*d, *ds)])
        .collect();
    Line::from(spans)
}

