use crate::app::App;
use crate::events::event::AppEvent::{
    AddTicketIdChar, JiraTicketListMove, JiraTicketListUpdate, JiraTicketMove, NewJiraTicketPopUp,
    RemoveTicket, RemoveTicketIdChar, SubmitTicketId, TicketRetrieved,
};
use crate::events::event::{AppEvent, Direction};
use crate::persistence::persister::JiraFile;
use crate::state::app::AppFocus;
use crate::utils::update_list_state;

pub fn handle_event(app: &mut App, app_event: AppEvent) {
    match app_event {
        JiraTicketListMove(direction) => {
            let list_len = app.state.jira.tickets.len();
            update_list_state::update_noneable_list(
                &mut app.state.jira.list_state,
                direction,
                list_len,
            );
        }
        NewJiraTicketPopUp => {
            app.state.jira.new_ticket_popup = true;
            app.state.focus = AppFocus::PopUp
        }
        AddTicketIdChar(char) => app.state.jira.add_char_to_ticket_id(char),
        RemoveTicketIdChar => {
            app.state.jira.remove_char_from_ticket_id();
        }
        SubmitTicketId => {
            if let Some(config) = app.config.jira.clone()
                && let Some(new_ticket_id) = app.state.jira.new_ticket_id.clone()
            {
                app.state.jira.new_ticket_popup = false;
                app.state.focus = AppFocus::Tool;

                let sender = app.event_sender.clone();

                app.jira_api.fetch_ticket(new_ticket_id, config, sender);
            }
        }
        TicketRetrieved(ticket_response) => {
            app.state.jira.add_ticket(ticket_response);
            app.state.jira.new_ticket_id = None;
            app.event_sender.send(JiraTicketListUpdate);
        }
        RemoveTicket => {
            app.state.jira.remove_ticket();
            let max_select = match app.state.jira.tickets.len() {
                0 | 1 => 0,
                value => value.saturating_sub(1),
            };

            if let Some(ticket_idx) = app.state.jira.list_state.selected()
                && ticket_idx > max_select
            {
                update_list_state::update_list(
                    &mut app.state.jira.list_state,
                    Direction::Up,
                    app.state.jira.tickets.len(),
                )
            }
            app.event_sender.send(JiraTicketListUpdate);
        }
        JiraTicketMove(direction) => {
            app.state.jira.swap_tickets(direction);
            app.event_sender.send(JiraTicketListUpdate);
        }
        JiraTicketListUpdate => {
            JiraFile::default()
                .write_jira(&app.state.jira.tickets)
                .expect("Failed to persist tickets");
        }
        _ => {}
    }
}
