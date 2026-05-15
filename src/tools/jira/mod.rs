use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::prelude::Rect;

use crate::client::jira::api::JiraApi;
use crate::config::model::{Config, Features};
use crate::event::events::AppEvent::{ActivityEvent, AppLog, RebuildToolList};
use crate::event::events::GenericEvent::OpenInBrowser;
use crate::event::events::JiraConfigEvent::{
    FormBackspace, FormDelete, FormEnd, FormHome, FormLeft, FormNextField, FormPrevField,
    FormRight, OpenEdit, SubmitConfig,
};
use crate::event::events::JiraEvent::{
    AddTicketIdChar, ListMove, NewTicket, RemoveTicket, RemoveTicketIdChar, ScanTickets,
    SubmitTicketId, TicketIdDelete, TicketIdEnd, TicketIdHome, TicketIdLeft, TicketIdRight,
    TicketListUpdate, TicketMove, TicketRetrieved,
};
use crate::event::events::{
    Direction, Event, GenericEvent, JiraConfigEvent, JiraEvent,
};
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{Editing, Tool as ToolCtx, ToolConfig, ToolConfigEditing};
use crate::input::key_event_map::KeyEventMap;
use crate::state::app::AppFocus;
use crate::state::jira::Jira;
use crate::state::jira_config::JiraConfigEditor;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;
use crate::tools::plugin::Plugin;
use crate::ui::widgets::config::jira as config_widget;
use crate::ui::widgets::tools::jira as widget;
use crate::utils::browser::open_link_in_browser;
use crate::utils::update_list_state;

const LOG_SOURCE: LogSource = LogSource::Jira;

pub struct JiraPlugin {
    state:       Jira,
    config_editor: JiraConfigEditor,
    jira_api:    Arc<dyn JiraApi>,
}

impl JiraPlugin {
    pub fn new(_config: &Config, jira_api: Arc<dyn JiraApi>) -> Self {
        Self {
            state:         Jira::new(),
            config_editor: JiraConfigEditor::new(),
            jira_api,
        }
    }

    fn handle_tool_event(&mut self, event: JiraEvent, ctx: &mut PluginContext) {
        match event {
            ListMove(direction) => {
                let list_len = self.state.tickets.len();
                update_list_state::update_noneable_list(
                    &mut self.state.list_state,
                    direction,
                    list_len,
                );
            }
            NewTicket => {
                self.state.new_ticket_id.clear();
                self.state.adding_ticket = true;
                *ctx.focus = AppFocus::JiraInput;
            }
            AddTicketIdChar(char) => self.state.add_char_to_ticket_id(char),
            RemoveTicketIdChar => {
                self.state.remove_char_from_ticket_id();
            }
            TicketIdLeft => self.state.new_ticket_id.move_left(),
            TicketIdRight => self.state.new_ticket_id.move_right(),
            TicketIdHome => self.state.new_ticket_id.home(),
            TicketIdEnd => self.state.new_ticket_id.end(),
            TicketIdDelete => self.state.new_ticket_id.delete_forward(),
            SubmitTicketId => {
                let ticket_id = self.state.new_ticket_id.value().to_string();
                if let Some(config) = ctx.config.jira.clone()
                    && !ticket_id.is_empty()
                {
                    self.state.adding_ticket = false;
                    self.state.new_ticket_id.clear();
                    *ctx.focus = AppFocus::Tool;
                    let sender = ctx.sender.clone();
                    self.jira_api.fetch_ticket(ticket_id, config, sender);
                }
            }
            TicketRetrieved(ticket_response) => {
                if self.state.tickets_pending_scan > 0 {
                    let changes = self.state.update_ticket_with_changes(ticket_response);
                    self.state.tickets_pending_scan = self.state.tickets_pending_scan.saturating_sub(1);
                    if let Some((id, change_msg)) = changes {
                        ctx.sender.send_app_event(ActivityEvent(id, change_msg));
                    }
                    if self.state.tickets_pending_scan == 0 {
                        ctx.sender.send_jira_event(TicketListUpdate);
                    }
                } else {
                    let ticket_id = ticket_response.key.clone();
                    self.state.add_ticket(ticket_response);
                    self.state.new_ticket_id.clear();
                    ctx.sender.send_app_event(ActivityEvent(ticket_id, "Added to watchlist".to_string()));
                    ctx.sender.send_jira_event(TicketListUpdate);
                }
            }
            RemoveTicket => {
                if let Some(idx) = self.state.list_state.selected()
                    && let Some(ticket) = self.state.tickets.get(idx)
                {
                    let id = ticket.id.clone();
                    ctx.sender.send_app_event(ActivityEvent(id, "Removed from watchlist".to_string()));
                }
                self.state.remove_ticket();
                if self.state.tickets.is_empty() {
                    self.state.list_state.select(None);
                } else if let Some(ticket_idx) = self.state.list_state.selected() {
                    let max_select = self.state.tickets.len().saturating_sub(1);
                    if ticket_idx > max_select {
                        update_list_state::update_list(
                            &mut self.state.list_state,
                            Direction::Up,
                            self.state.tickets.len(),
                        );
                    }
                }
                ctx.sender.send_jira_event(TicketListUpdate);
            }
            TicketMove(direction) => {
                self.state.swap_tickets(direction);
                ctx.sender.send_jira_event(TicketListUpdate);
            }
            TicketListUpdate => {
                if let Err(e) = self.state.jira_file.write_jira(&self.state.tickets) {
                    ctx.sender.send_app_event(AppLog(
                        LogEntry::new(
                            LogLevel::Error,
                            LOG_SOURCE,
                            "Unable to persist Jira tickets",
                        )
                        .with_detail(e.to_string()),
                    ));
                }
            }
            ScanTickets => {
                if self.state.tickets.is_empty() || self.state.tickets_pending_scan > 0 {
                    if self.state.tickets_pending_scan > 0 {
                        ctx.sender.send_app_event(AppLog(LogEntry::new(
                            LogLevel::Warning,
                            LOG_SOURCE,
                            "Ticket scan skipped — previous scan still running",
                        )));
                    }
                    return;
                }
                if let Some(config) = &ctx.config.jira {
                    let count = self.state.tickets.len();
                    ctx.sender.send_app_event(AppLog(LogEntry::new(
                        LogLevel::Info,
                        LOG_SOURCE,
                        format!("Ticket scan started — {} tickets", count),
                    )));
                    self.state.tickets_pending_scan = self.state.tickets.len();
                    for t in self.state.tickets.iter() {
                        self.jira_api.fetch_ticket(
                            t.id.clone(),
                            config.clone(),
                            ctx.sender.clone(),
                        );
                    }
                }
            }
        }
    }

    fn handle_config_event(&mut self, event: JiraConfigEvent, ctx: &mut PluginContext) {
        match event {
            OpenEdit => {
                self.config_editor.open_form(ctx.config.jira.as_ref());
            }
            FormNextField => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field = p.active_field.next();
                }
            }
            FormPrevField => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field = p.active_field.prev();
                }
            }
            crate::event::events::JiraConfigEvent::FormChar(c) => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field_mut().insert(c);
                }
            }
            FormBackspace => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field_mut().backspace();
                }
            }
            FormLeft => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field_mut().move_left();
                }
            }
            FormRight => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field_mut().move_right();
                }
            }
            FormHome => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field_mut().home();
                }
            }
            FormEnd => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field_mut().end();
                }
            }
            FormDelete => {
                if let Some(p) = &mut self.config_editor.form {
                    p.active_field_mut().delete_forward();
                }
            }
            SubmitConfig => {
                if let Some(form) = self.config_editor.form.take() {
                    if form.is_empty() {
                        ctx.config.jira = None;
                    } else {
                        ctx.config.jira = Some(crate::config::model::JiraConfig {
                            url: form.url.value().trim().to_string(),
                            email: form.email.value().trim().to_string(),
                            token: form.token.value().trim().to_string(),
                        });
                    }
                    ctx.config.enforce_feature_invariants();
                    ctx.sender.send_app_event(RebuildToolList);
                    let _ = ctx.config_loader.write_config(ctx.config);
                }
            }
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
