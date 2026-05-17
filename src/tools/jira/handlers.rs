use super::JiraPlugin;
use crate::event::events::AppEvent::{ActivityEvent, AppLog, RebuildToolList};
use crate::event::events::JiraConfigEvent::{
    FormBackspace, FormDelete, FormEnd, FormHome, FormLeft, FormNextField, FormPrevField,
    FormRight, OpenEdit, SubmitConfig,
};
use crate::event::events::JiraEvent::{
    AddTicketIdChar, ListMove, NewTicket, RemoveTicket, RemoveTicketIdChar, ScanTickets,
    SubmitTicketId, TicketIdDelete, TicketIdEnd, TicketIdHome, TicketIdLeft, TicketIdRight,
    TicketListUpdate, TicketMove, TicketRetrieved,
};
use crate::event::events::{Direction, JiraConfigEvent, JiraEvent};
use crate::state::app::AppFocus;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::tools::context::PluginContext;
use crate::utils::update_list_state;

const LOG_SOURCE: LogSource = LogSource::Jira;

impl JiraPlugin {
    pub(super) fn handle_tool_event(&mut self, event: JiraEvent, ctx: &mut PluginContext) {
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

    pub(super) fn handle_config_event(&mut self, event: JiraConfigEvent, ctx: &mut PluginContext) {
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
