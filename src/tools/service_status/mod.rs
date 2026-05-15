use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::prelude::Rect;

use crate::client::healthcheck::api::HealthcheckApi;
use crate::config::model::{Config, Features};
use crate::environment::Environment::{Preproduction, Production, Staging};
use crate::event::events::AppEvent::{ActivityEvent, AppLog, RebuildToolList};
use crate::event::events::ServiceStatusConfigEvent::{
    FormBackspace, FormDelete, FormEnd, FormHome, FormLeft, FormNextField, FormRight,
    ListMove as ConfigListMove, OpenAddService, OpenEditService, PrevField, RemoveService,
    SubmitConfig,
};
use crate::event::events::ServiceStatusEvent::{
    GetCommitRefErrored, GetCommitRefOk, ListMove, Scan, ScanServiceEnv,
};
use crate::event::events::{
    Event, GenericEvent, ServiceStatusConfigEvent, ServiceStatusEvent,
};
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{Editing, Tool as ToolCtx, ToolConfig};
use crate::input::key_event_map::KeyEventMap;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::service_status::{CommitRefStatus, ServiceStatus};
use crate::state::service_status_config::ServiceStatusConfigEditor;
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;
use crate::tools::plugin::Plugin;
use crate::ui::widgets::config::service_status as config_widget;
use crate::ui::widgets::tools::service_status as widget;
use crate::utils::browser::open_link_in_browser;
use crate::utils::string_copy::copy_to_clipboard;

const LOG_SOURCE: LogSource = LogSource::ServiceStatus;

pub struct ServiceStatusPlugin {
    state:         ServiceStatus,
    config_editor: ServiceStatusConfigEditor,
    healthcheck_api: Arc<dyn HealthcheckApi>,
}

impl ServiceStatusPlugin {
    pub fn new(config: &Config, healthcheck_api: Arc<dyn HealthcheckApi>) -> Self {
        Self {
            state:         ServiceStatus::new(config.servicestatus.len()),
            config_editor: ServiceStatusConfigEditor::new(),
            healthcheck_api,
        }
    }

    fn handle_tool_event(&mut self, event: ServiceStatusEvent, ctx: &mut PluginContext) {
        match event {
            ListMove(direction) => {
                let len = self.state.services.len();
                let table_state = &mut self.state.table_state;

                if len == 0 {
                    table_state.select(None);
                    return;
                }

                match direction {
                    crate::event::events::Direction::Up => {
                        if table_state.selected().unwrap_or(0) > 0 {
                            table_state.select_previous();
                        } else {
                            table_state.select(None);
                        }
                    }
                    crate::event::events::Direction::Down => {
                        let selected = table_state.selected().unwrap_or(0);
                        let max = len.saturating_sub(1);
                        if selected < max {
                            table_state.select_next();
                        } else {
                            table_state.select(Some(max));
                        }
                    }
                }
            }
            Scan => {
                let len = self.state.services.len();
                let sender = ctx.sender.clone();
                sender.send_app_event(AppLog(LogEntry::new(
                    LogLevel::Info,
                    LOG_SOURCE,
                    format!("Scan started — {} services × 3 environments", len),
                )));
                for service_idx in 0..len {
                    sender.send_service_status_event(ScanServiceEnv(service_idx, Staging));
                    sender.send_service_status_event(ScanServiceEnv(service_idx, Preproduction));
                    sender.send_service_status_event(ScanServiceEnv(service_idx, Production));
                }
            }
            ScanServiceEnv(service_idx, env) => {
                self.state.set_commit_fetching(service_idx, &env);
                let sender = ctx.sender.clone();
                let config = ctx.config.servicestatus.clone();
                self.healthcheck_api.commit_ref(service_idx, env, config.into(), sender);
            }
            GetCommitRefOk(commit, service_idx, env) => {
                let old_status = self.state.services[service_idx].commit_ref_status();
                self.state.set_commit_ok(service_idx, &env, commit);
                let new_status = self.state.services[service_idx].commit_ref_status();

                if old_status != new_status
                    && !matches!(new_status, CommitRefStatus::CommitMissing)
                    && let Some(svc_cfg) = ctx.config.servicestatus.get(service_idx)
                {
                    let msg = status_activity_message(&new_status);
                    ctx.sender.send_app_event(ActivityEvent(svc_cfg.name.clone(), msg));
                }
            }
            GetCommitRefErrored(error, service_idx, env) => {
                self.state.set_commit_error(service_idx, &env, error.clone());
                if let Some(svc_cfg) = ctx.config.servicestatus.get(service_idx) {
                    let env_label = env.to_string().to_lowercase();
                    ctx.sender.send_app_event(AppLog(LogEntry::new(
                        LogLevel::Warning,
                        LOG_SOURCE,
                        format!("{}/{}: {}", svc_cfg.name, env_label, friendly_error(&error)),
                    )));
                }
            }
        }
    }

    fn handle_config_event(&mut self, event: ServiceStatusConfigEvent, ctx: &mut PluginContext) {
        match event {
            ConfigListMove(direction) => {
                let len = ctx.config.servicestatus.len();
                let state = &mut self.config_editor.table_state;
                if len == 0 {
                    state.select(None);
                } else {
                    match direction {
                        crate::event::events::Direction::Up => match state.selected() {
                            None | Some(0) => state.select(None),
                            _ => state.select_previous(),
                        },
                        crate::event::events::Direction::Down => {
                            let next = state.selected().map(|i| i + 1).unwrap_or(0);
                            state.select(Some(next.min(len - 1)));
                        }
                    }
                }
            }
            OpenAddService => {
                self.config_editor.open_form();
            }
            OpenEditService => {
                if let Some(idx) = self.config_editor.table_state.selected()
                    && let Some(svc) = ctx.config.servicestatus.get(idx)
                {
                    let svc = svc.clone();
                    self.config_editor.open_edit_form(idx, &svc);
                }
            }
            FormNextField => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field = form.active_field.next();
                }
            }
            PrevField => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field = form.active_field.prev();
                }
            }
            crate::event::events::ServiceStatusConfigEvent::FormChar(c) => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().insert(c);
                }
            }
            FormBackspace => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().backspace();
                }
            }
            FormLeft => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().move_left();
                }
            }
            FormRight => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().move_right();
                }
            }
            FormHome => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().home();
                }
            }
            FormEnd => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().end();
                }
            }
            FormDelete => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().delete_forward();
                }
            }
            SubmitConfig => {
                if let Some(form) = self.config_editor.form.take()
                    && form.is_valid()
                {
                    let service = crate::config::model::ServiceStatusConfig {
                        name: form.name.value().trim().to_string(),
                        staging: form.staging.value().trim().to_string(),
                        preproduction: form.preprod.value().trim().to_string(),
                        production: form.prod.value().trim().to_string(),
                        repo: form.repo.value().trim().to_string(),
                    };
                    if let Some(idx) = form.edit_index {
                        if let Some(existing) = ctx.config.servicestatus.get_mut(idx) {
                            *existing = service;
                        }
                    } else {
                        ctx.config.servicestatus.push(service);
                    }
                    self.state = ServiceStatus::new(ctx.config.servicestatus.len());
                    let _ = ctx.config_loader.write_config(ctx.config);
                }
            }
            RemoveService => {
                if let Some(idx) = self.config_editor.table_state.selected()
                    && idx < ctx.config.servicestatus.len()
                {
                    ctx.config.servicestatus.remove(idx);
                    self.state = ServiceStatus::new(ctx.config.servicestatus.len());
                    let new_len = ctx.config.servicestatus.len();
                    if new_len == 0 {
                        self.config_editor.table_state.select(None);
                        ctx.config.enforce_feature_invariants();
                        ctx.sender.send_app_event(RebuildToolList);
                    } else {
                        let clamped = idx.min(new_len - 1);
                        self.config_editor.table_state.select(Some(clamped));
                    }
                    let _ = ctx.config_loader.write_config(ctx.config);
                }
            }
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

fn status_activity_message(status: &CommitRefStatus) -> String {
    match status {
        CommitRefStatus::AllMatches => "Now in sync across all environments".to_string(),
        CommitRefStatus::StagingPreprodMatch => {
            "Ready for production — staging and preprod match".to_string()
        }
        CommitRefStatus::PreprodProdMatch => "New version in the deployment pipeline".to_string(),
        CommitRefStatus::NothingMatches => "Environments are out of sync".to_string(),
        CommitRefStatus::CommitMissing => {
            "Commit errors detected — may require maintenance".to_string()
        }
    }
}

fn friendly_error(raw: &str) -> String {
    if raw.contains("timed out") {
        "Request timed out — check VPN connection".to_string()
    } else if raw.contains("503") || raw.contains("Service Unavailable") {
        "Service unavailable".to_string()
    } else {
        raw.to_string()
    }
}
