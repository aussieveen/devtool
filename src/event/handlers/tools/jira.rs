use crate::app::App;
use crate::error::model::Error;
use crate::event::event::AppEvent::{ActivityEvent, AppLog, SystemError};
use crate::event::event::{Direction, GenericEvent, JiraEvent};
use crate::event::event::GenericEvent::OpenInBrowser;
use crate::event::event::JiraEvent::{
    AddTicketIdChar, ListMove, TicketListUpdate, TicketMove, NewTicket,
    RemoveTicket, RemoveTicketIdChar, ScanTickets, SubmitTicketId, TicketRetrieved
};
use crate::state::app::AppFocus;
use crate::state::log::LogLevel;
use crate::utils::browser::open_link_in_browser;
use crate::utils::update_list_state;

pub fn handle_event(app: &mut App, event: JiraEvent) {
    match event {
        ListMove(direction) => {
            let list_len = app.state.jira.tickets.len();
            update_list_state::update_noneable_list(
                &mut app.state.jira.list_state,
                direction,
                list_len,
            );
        }
        NewTicket => {
            app.state.jira.adding_ticket = true;
            app.state.focus = AppFocus::JiraInput
        }
        AddTicketIdChar(char) => app.state.jira.add_char_to_ticket_id(char),
        RemoveTicketIdChar => {
            app.state.jira.remove_char_from_ticket_id();
        }
        SubmitTicketId => {
            if let Some(config) = app.config.jira.clone()
                && let Some(new_ticket_id) = app.state.jira.new_ticket_id.clone()
            {
                app.state.jira.adding_ticket = false;
                app.state.focus = AppFocus::Tool;

                let sender = app.event_sender.clone();

                app.jira_api.fetch_ticket(new_ticket_id, config, sender);
            }
        }
        TicketRetrieved(ticket_response) => {
            if app.state.jira.tickets_pending_scan > 0 {
                let changes = app.state.jira.update_ticket_with_changes(ticket_response);
                app.state.jira.tickets_pending_scan =
                    app.state.jira.tickets_pending_scan.saturating_sub(1);
                if let Some((id, change_msg)) = changes {
                    app.event_sender.send_app_event(ActivityEvent(id, change_msg));
                }
                if app.state.jira.tickets_pending_scan == 0 {
                    app.event_sender.send_jira_event(TicketListUpdate)
                }
            } else {
                let ticket_id = ticket_response.key.clone();
                app.state.jira.add_ticket(ticket_response);
                app.state.jira.new_ticket_id = None;
                app.event_sender
                    .send_app_event(ActivityEvent(ticket_id, "Added to watchlist".to_string()));
                app.event_sender.send_jira_event(TicketListUpdate);
            }
        }
        RemoveTicket => {
            if let Some(idx) = app.state.jira.list_state.selected()
                && let Some(ticket) = app.state.jira.tickets.get(idx)
            {
                let id = ticket.id.clone();
                app.event_sender
                    .send_app_event(ActivityEvent(id, "Removed from watchlist".to_string()));
            }
            app.state.jira.remove_ticket();
            if app.state.jira.tickets.is_empty() {
                app.state.jira.list_state.select(None);
            } else if let Some(ticket_idx) = app.state.jira.list_state.selected() {
                let max_select = app.state.jira.tickets.len().saturating_sub(1);
                if ticket_idx > max_select {
                    update_list_state::update_list(
                        &mut app.state.jira.list_state,
                        Direction::Up,
                        app.state.jira.tickets.len(),
                    )
                }
            }
            app.event_sender.send_jira_event(TicketListUpdate);
        }
        TicketMove(direction) => {
            app.state.jira.swap_tickets(direction);
            app.event_sender.send_jira_event(TicketListUpdate);
        }
        TicketListUpdate => {
            if let Err(e) = app.state.jira.jira_file.write_jira(&app.state.jira.tickets) {
                let sender = app.event_sender.clone();
                sender.send_app_event(SystemError(Error {
                    title: "Unable to persist jira tickets".to_string(),
                    originating_event: "JiraTicketListUpdate".to_string(),
                    tool: "Jira".to_string(),
                    description: e.to_string(),
                }))
            }
        }
        ScanTickets => {
            // If there are no tickets or a previous scan is still running
            if app.state.jira.tickets.is_empty() || app.state.jira.tickets_pending_scan > 0 {
                if app.state.jira.tickets_pending_scan > 0 {
                    app.event_sender.send_app_event(AppLog(
                        LogLevel::Warning,
                        "jira".to_string(),
                        "Ticket scan skipped — previous scan still running".to_string(),
                    ));
                }
                return;
            }
            if let Some(config) = app.config.jira.clone() {
                let count = app.state.jira.tickets.len();
                app.event_sender.send_app_event(AppLog(
                    LogLevel::Info,
                    "jira".to_string(),
                    format!("Ticket scan started — {} tickets", count),
                ));
                app.state.jira.tickets_pending_scan = app.state.jira.tickets.len();
                for t in app.state.jira.tickets.iter() {
                    app.jira_api.fetch_ticket(
                        t.id.clone(),
                        config.clone(),
                        app.event_sender.clone(),
                    );
                }
            }
        }
    }
}

pub fn handle_generic_event(app: &mut App, event: GenericEvent){
    match event {
        OpenInBrowser => {
            if let Some(jira_ticket_idx) = app.state.jira.list_state.selected()
                && let Some(config) = app.config.jira.clone()
            {
                let link = format!(
                    "{}/browse/{}",
                    config.url, app.state.jira.tickets[jira_ticket_idx].id
                );
                if let Err(e) = open_link_in_browser(link.as_str()) {
                    app.event_sender.send_app_event(AppLog(
                        LogLevel::Warning,
                        "jira".to_string(),
                        format!("Open in browser failed: {}", e),
                    ));
                }
            }
        }
        _ => {}
    }
}
