pub mod state;
pub mod config_editor;
pub(super) mod widget;
pub(super) mod config_widget;
mod handlers;

use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::prelude::Rect;

use crate::client::healthcheck::api::HealthcheckApi;
use crate::config::model::{Config, Features};
use crate::event::events::AppEvent::AppLog;
use crate::event::events::{Event, GenericEvent, ServiceStatusConfigEvent, ServiceStatusEvent};
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{Editing, Tool as ToolCtx, ToolConfig};
use crate::input::key_event_map::KeyEventMap;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;
use crate::tools::plugin::Plugin;
use crate::utils::browser::open_link_in_browser;
use crate::utils::string_copy::copy_to_clipboard;
use self::state::ServiceStatus;
use self::config_editor::ServiceStatusConfigEditor;

const LOG_SOURCE: LogSource = LogSource::ServiceStatus;

pub struct ServiceStatusPlugin {
    pub(super) state:           ServiceStatus,
    pub(super) config_editor:   ServiceStatusConfigEditor,
    pub(super) healthcheck_api: Arc<dyn HealthcheckApi>,
}

impl ServiceStatusPlugin {
    pub fn new(config: &Config, healthcheck_api: Arc<dyn HealthcheckApi>) -> Self {
        Self {
            state:           ServiceStatus::new(config.servicestatus.len()),
            config_editor:   ServiceStatusConfigEditor::new(),
            healthcheck_api,
        }
    }

    fn link_url(&self, config: &Config) -> Option<String> {
        if !self.state.has_link() {
            return None;
        }
        let service_idx = self.state.selected_service_idx()?;
        self.state.link(&config.servicestatus[service_idx].repo)
    }
}

impl Plugin for ServiceStatusPlugin {
    fn id(&self)           -> Tool        { Tool::ServiceStatus }
    fn title(&self)        -> &'static str { "Service Status" }
    fn menu_entry(&self)   -> &'static str { "Service Status" }
    fn config_title(&self) -> &'static str { " Service Status — Config " }

    fn has_min_config(&self, config: &Config) -> bool {
        !config.servicestatus.is_empty()
    }
    fn is_enabled(&self, features: &Features) -> bool { features.service_status }
    fn apply_feature_flag(&self, features: &mut Features, enabled: bool) {
        features.service_status = enabled;
    }

    fn register_bindings(&self, map: &mut KeyEventMap) {
        use crate::event::events::Direction;

        map.add_static(ToolCtx(Tool::ServiceStatus), KeyCode::Left, KeyModifiers::NONE,
            Event::Generic(crate::event::events::GenericEvent::SetFocus(crate::state::app::AppFocus::List)));
        map.add_static(ToolCtx(Tool::ServiceStatus), KeyCode::Down, KeyModifiers::NONE,
            Event::ServiceStatus(ServiceStatusEvent::ListMove(Direction::Down)));
        map.add_static(ToolCtx(Tool::ServiceStatus), KeyCode::Up, KeyModifiers::NONE,
            Event::ServiceStatus(ServiceStatusEvent::ListMove(Direction::Up)));
        map.add_static(ToolCtx(Tool::ServiceStatus), KeyCode::Char('s'), KeyModifiers::NONE,
            Event::ServiceStatus(ServiceStatusEvent::Scan));

        // Config
        map.add_static(ToolConfig(Tool::ServiceStatus), KeyCode::Down, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::ListMove(Direction::Down)));
        map.add_static(ToolConfig(Tool::ServiceStatus), KeyCode::Up, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::ListMove(Direction::Up)));
        map.add_static(ToolConfig(Tool::ServiceStatus), KeyCode::Char('a'), KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::OpenAddService));
        map.add_static(ToolConfig(Tool::ServiceStatus), KeyCode::Char('e'), KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::OpenEditService));
        map.add_static(ToolConfig(Tool::ServiceStatus), KeyCode::Char('x'), KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::RemoveService));
        map.add_static(ToolConfig(Tool::ServiceStatus), KeyCode::Left, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));
        map.add_static(ToolConfig(Tool::ServiceStatus), KeyCode::Esc, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));

        // Config editing
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Esc, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Enter, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::SubmitConfig));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Backspace, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormBackspace));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Left, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormLeft));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Right, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormRight));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Home, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormHome));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::End, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormEnd));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Delete, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormDelete));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Down, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormNextField));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Up, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::PrevField));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::Tab, KeyModifiers::NONE,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormNextField));
        map.add_static(Editing(Tool::ServiceStatus), KeyCode::BackTab, KeyModifiers::SHIFT,
            Event::ServiceStatusConfig(ServiceStatusConfigEvent::PrevField));
        map.add_dynamic(Editing(Tool::ServiceStatus), service_status_form_char);
    }

    fn key_contexts(&self) -> Vec<KeyContext> {
        vec![ToolCtx(Tool::ServiceStatus)]
    }

    fn config_key_contexts(&self) -> Vec<KeyContext> {
        if self.config_editor.has_open_form() {
            vec![Editing(Tool::ServiceStatus)]
        } else {
            vec![ToolConfig(Tool::ServiceStatus)]
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        widget::render(frame, area, &mut self.state, &config.servicestatus);
    }

    fn render_config(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        config_widget::render(frame, area, &mut self.config_editor, &config.servicestatus);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut PluginContext) -> bool {
        match event {
            Event::ServiceStatus(e) => { self.handle_tool_event(e.clone(), ctx); true }
            Event::ServiceStatusConfig(e) => { self.handle_config_event(e.clone(), ctx); true }
            _ => false,
        }
    }

    fn handle_generic_event(&mut self, event: &GenericEvent, ctx: &mut PluginContext) -> bool {
        match event {
            GenericEvent::CopyToClipboard => {
                if let Some(link) = self.link_url(ctx.config)
                    && let Err(e) = copy_to_clipboard(link.as_str())
                {
                    ctx.sender.send_app_event(AppLog(LogEntry::new(
                        LogLevel::Warning,
                        LOG_SOURCE,
                        format!("Copy to clipboard failed: {e}"),
                    )));
                }
                true
            }
            GenericEvent::OpenInBrowser => {
                if let Some(link) = self.link_url(ctx.config)
                    && let Err(e) = open_link_in_browser(link.as_str())
                {
                    ctx.sender.send_app_event(AppLog(LogEntry::new(
                        LogLevel::Warning,
                        LOG_SOURCE,
                        format!("Open in browser failed: {e}"),
                    )));
                }
                true
            }
            _ => false,
        }
    }

    fn has_open_form(&self) -> bool { self.config_editor.has_open_form() }
    fn close_form(&mut self)        { self.config_editor.close_form(); }

    fn tool_hints(&self) -> (ratatui::text::Line<'static>, ratatui::text::Line<'static>) {
        use crate::ui::styles::{key_desc_style, key_style};
        use ratatui::text::{Line, Span};
        let k = key_style();
        let d = key_desc_style();
        let line1 = Line::from(vec![
            Span::styled("[↑↓←→]", k.clone()), Span::styled(" Navigate  ", d.clone()),
            Span::styled("[s]", k.clone()), Span::styled(" Scan  ", d.clone()),
            Span::styled("[q]", k.clone()), Span::styled(" Quit", d.clone()),
        ]);
        let line2 = if self.state.has_link() {
            Line::from(vec![
                Span::styled("[o]", k.clone()), Span::styled(" Open in browser  ", d.clone()),
                Span::styled("[c]", k.clone()), Span::styled(" Copy url  ", d.clone()),
            ])
        } else {
            Line::from("")
        };
        (line1, line2)
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
        let k = key_style(); let d = key_desc_style();
        let line2 = if self.config_editor.table_state.selected().is_some() {
            Line::from(vec![
                Span::styled("[e]", k.clone()), Span::styled(" Edit  ", d.clone()),
                Span::styled("[x]", k.clone()), Span::styled(" Remove  ", d.clone()),
            ])
        } else { Line::from("") };
        (Line::from(vec![
            Span::styled("[↑↓←→]", k.clone()), Span::styled(" Navigate  ", d.clone()),
            Span::styled("[a]", k.clone()), Span::styled(" Add  ", d.clone()),
            Span::styled("[q]", k.clone()), Span::styled(" Quit", d.clone()),
        ]), line2)
    }
}

fn service_status_form_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| Event::ServiceStatusConfig(ServiceStatusConfigEvent::FormChar(c)))
}
