pub mod state;
pub mod config_editor;
pub(super) mod widget;
pub(super) mod config_widget;
mod handlers;

use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::prelude::Rect;

use crate::client::jira::api::JiraApi;
use crate::config::model::{Config, Features};
use crate::event::events::AppEvent::AppLog;
use crate::event::events::GenericEvent::OpenInBrowser;
use crate::event::events::{Event, GenericEvent, JiraConfigEvent, JiraEvent};
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{Editing, Tool as ToolCtx, ToolConfig, ToolConfigEditing};
use crate::input::key_event_map::KeyEventMap;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;
use crate::tools::plugin::Plugin;
use crate::utils::browser::open_link_in_browser;
use self::state::Jira;
use self::config_editor::JiraConfigEditor;

const LOG_SOURCE: LogSource = LogSource::Jira;

pub struct JiraPlugin {
    pub(super) state:         Jira,
    pub(super) config_editor: JiraConfigEditor,
    pub(super) jira_api:      Arc<dyn JiraApi>,
}

impl JiraPlugin {
    pub fn new(_config: &Config, jira_api: Arc<dyn JiraApi>) -> Self {
        Self {
            state:         Jira::new(),
            config_editor: JiraConfigEditor::new(),
            jira_api,
        }
    }
}

impl Plugin for JiraPlugin {
    fn id(&self)           -> Tool        { Tool::Jira }
    fn title(&self)        -> &'static str { "Jira" }
    fn menu_entry(&self)   -> &'static str { "Jira" }
    fn config_title(&self) -> &'static str { " Jira — Config " }

    fn has_min_config(&self, config: &Config) -> bool {
        config.jira.is_some()
    }
    fn is_enabled(&self, features: &Features) -> bool { features.jira }
    fn apply_feature_flag(&self, features: &mut Features, enabled: bool) {
        features.jira = enabled;
    }

    fn register_bindings(&self, map: &mut KeyEventMap) {
        use crate::event::events::Direction;
        // Tool
        map.add_static(ToolCtx(Tool::Jira), KeyCode::Left, KeyModifiers::NONE,
            Event::Generic(crate::event::events::GenericEvent::SetFocus(crate::state::app::AppFocus::List)));
        map.add_static(ToolCtx(Tool::Jira), KeyCode::Up, KeyModifiers::NONE,
            Event::Jira(JiraEvent::ListMove(Direction::Up)));
        map.add_static(ToolCtx(Tool::Jira), KeyCode::Down, KeyModifiers::NONE,
            Event::Jira(JiraEvent::ListMove(Direction::Down)));
        map.add_static(ToolCtx(Tool::Jira), KeyCode::Up, KeyModifiers::SHIFT,
            Event::Jira(JiraEvent::TicketMove(Direction::Up)));
        map.add_static(ToolCtx(Tool::Jira), KeyCode::Down, KeyModifiers::SHIFT,
            Event::Jira(JiraEvent::TicketMove(Direction::Down)));
        map.add_static(ToolCtx(Tool::Jira), KeyCode::Char('a'), KeyModifiers::NONE,
            Event::Jira(JiraEvent::NewTicket));
        map.add_static(ToolCtx(Tool::Jira), KeyCode::Char('x'), KeyModifiers::NONE,
            Event::Jira(JiraEvent::RemoveTicket));

        // Config
        map.add_static(ToolConfig(Tool::Jira), KeyCode::Char('e'), KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::OpenEdit));
        map.add_static(ToolConfig(Tool::Jira), KeyCode::Left, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));
        map.add_static(ToolConfig(Tool::Jira), KeyCode::Esc, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));

        // Config editing (uses ToolConfigEditing, not Editing, for Jira)
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Esc, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Enter, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::SubmitConfig));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Backspace, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormBackspace));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Left, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormLeft));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Right, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormRight));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Home, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormHome));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::End, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormEnd));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Delete, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormDelete));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Down, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormNextField));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Up, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormPrevField));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::Tab, KeyModifiers::NONE,
            Event::JiraConfig(JiraConfigEvent::FormNextField));
        map.add_static(ToolConfigEditing(Tool::Jira), KeyCode::BackTab, KeyModifiers::SHIFT,
            Event::JiraConfig(JiraConfigEvent::FormPrevField));
        map.add_dynamic(ToolConfigEditing(Tool::Jira), jira_config_form_char);

        // Jira input (add ticket) — uses Editing(Jira)
        map.add_static(Editing(Tool::Jira), KeyCode::Backspace, KeyModifiers::NONE,
            Event::Jira(JiraEvent::RemoveTicketIdChar));
        map.add_static(Editing(Tool::Jira), KeyCode::Left, KeyModifiers::NONE,
            Event::Jira(JiraEvent::TicketIdLeft));
        map.add_static(Editing(Tool::Jira), KeyCode::Right, KeyModifiers::NONE,
            Event::Jira(JiraEvent::TicketIdRight));
        map.add_static(Editing(Tool::Jira), KeyCode::Home, KeyModifiers::NONE,
            Event::Jira(JiraEvent::TicketIdHome));
        map.add_static(Editing(Tool::Jira), KeyCode::End, KeyModifiers::NONE,
            Event::Jira(JiraEvent::TicketIdEnd));
        map.add_static(Editing(Tool::Jira), KeyCode::Delete, KeyModifiers::NONE,
            Event::Jira(JiraEvent::TicketIdDelete));
        map.add_static(Editing(Tool::Jira), KeyCode::Enter, KeyModifiers::NONE,
            Event::Jira(JiraEvent::SubmitTicketId));
        map.add_dynamic(Editing(Tool::Jira), add_ticket_id_char);
    }

    fn key_contexts(&self) -> Vec<KeyContext> {
        vec![ToolCtx(Tool::Jira)]
    }

    fn config_key_contexts(&self) -> Vec<KeyContext> {
        if self.config_editor.has_open_form() {
            vec![ToolConfigEditing(Tool::Jira)]
        } else {
            vec![ToolConfig(Tool::Jira)]
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &Config) {
        widget::render(frame, area, &mut self.state);
    }

    fn render_config(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        config_widget::render(frame, area, &mut self.config_editor, config.jira.as_ref());
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut PluginContext) -> bool {
        match event {
            Event::Jira(e) => { self.handle_tool_event(e.clone(), ctx); true }
            Event::JiraConfig(e) => { self.handle_config_event(e.clone(), ctx); true }
            _ => false,
        }
    }

    fn handle_generic_event(&mut self, event: &GenericEvent, ctx: &mut PluginContext) -> bool {
        if *event == OpenInBrowser
            && let Some(jira_ticket_idx) = self.state.list_state.selected()
            && let Some(config) = ctx.config.jira.clone()
        {
            let link = format!(
                "{}/browse/{}",
                config.url, self.state.tickets[jira_ticket_idx].id
            );
            if let Err(e) = open_link_in_browser(link.as_str()) {
                ctx.sender.send_app_event(AppLog(LogEntry::new(
                    LogLevel::Warning,
                    LOG_SOURCE,
                    format!("Open in browser failed: {e}"),
                )));
            }
            true
        } else {
            false
        }
    }

    fn has_open_form(&self) -> bool { self.config_editor.has_open_form() }
    fn close_form(&mut self)        { self.config_editor.close_form(); }

    fn tool_hints(&self) -> (ratatui::text::Line<'static>, ratatui::text::Line<'static>) {
        use crate::ui::styles::{key_desc_style, key_style};
        use ratatui::text::{Line, Span};
        let k = key_style();
        let d = key_desc_style();
        let line2 = if self.state.list_state.selected().is_some() {
            Line::from(vec![
                Span::styled("[x]", k.clone()), Span::styled(" Remove  ", d.clone()),
                Span::styled("[o]", k.clone()), Span::styled(" Open in browser  ", d.clone()),
                Span::styled("[shift+↑↓]", k.clone()), Span::styled(" Move  ", d.clone()),
            ])
        } else { Line::from("") };
        (Line::from(vec![
            Span::styled("[↑↓←→]", k.clone()), Span::styled(" Navigate  ", d.clone()),
            Span::styled("[a]", k.clone()), Span::styled(" Add  ", d.clone()),
            Span::styled("[q]", k.clone()), Span::styled(" Quit", d.clone()),
        ]), line2)
    }

    fn config_hints(&self) -> (ratatui::text::Line<'static>, ratatui::text::Line<'static>) {
        use crate::ui::styles::{key_desc_style, key_style};
        use ratatui::text::{Line, Span};
        if self.config_editor.has_open_form() {
            let k = key_style(); let d = key_desc_style();
            return (
                Line::from(vec![
                    Span::styled("[return]", k.clone()), Span::styled(" Save  ", d.clone()),
                    Span::styled("[tab]", k.clone()), Span::styled(" Next field  ", d.clone()),
                    Span::styled("[↑↓]", k.clone()), Span::styled(" Navigate fields  ", d.clone()),
                ]),
                Line::from(vec![Span::styled("[esc]", key_style()), Span::styled(" Cancel  ", key_desc_style())]),
            );
        }
        (Line::from(vec![
            Span::styled("[e]", key_style()), Span::styled(" Edit  ", key_desc_style()),
            Span::styled("[q]", key_style()), Span::styled(" Quit", key_desc_style()),
        ]), Line::from(""))
    }
}

fn add_ticket_id_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| Event::Jira(JiraEvent::AddTicketIdChar(c)))
}

fn jira_config_form_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| Event::JiraConfig(JiraConfigEvent::FormChar(c)))
}
