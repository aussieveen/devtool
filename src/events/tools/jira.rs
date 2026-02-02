use std::error::Error;
use crate::app::App;
use crate::client::jira_client;
use crate::client::jira_client::TicketResponse;
use crate::config::JiraConfig;
use crate::events::event::{AppEvent, Direction};
use crate::events::event::AppEvent::{AddTicketIdChar, NewJiraTicketPopUp, JiraTicketListMove, RemoveTicketIdChar, SubmitTicketId, TicketRetrieved, RemoveTicket, JiraTicketMove};
use crate::persistence::write_jira_tickets;
use crate::state::app::AppFocus;
use crate::utils::update_list_state;

pub fn handle_event(app: &mut App, app_event: AppEvent){
    match (app_event) {
        JiraTicketListMove(direction) => {
            let list_len = app.state.jira.tickets.len();
            update_list_state::update_noneable_list(&mut app.state.jira.list_state, direction, list_len);
        }
        NewJiraTicketPopUp => {
            app.state.jira.set_new_ticket_popup(true);
            app.state.focus = AppFocus::PopUp
        }
        AddTicketIdChar(char) => {
            app.state.jira.add_char_to_ticket_id(char)
        }
        RemoveTicketIdChar => {
            app.state.jira.remove_char_from_ticket_id();
        }
        SubmitTicketId => {
            if let Some(config) = app.config.jira.clone()
                && let Some(new_ticket_id) = app.state.jira.new_ticket_id.clone()
            {
                app.state.jira.set_new_ticket_popup(false);
                app.state.focus = AppFocus::Tool;

                let sender = app.event_sender.clone();

                tokio::spawn(async move {
                    match get_ticket(&new_ticket_id, &config).await {
                        Ok(ticket) => {
                            sender.send(TicketRetrieved(ticket));
                        }
                        Err(_err) => {
                            todo!()
                        }
                    }
                });
            }
        }
        TicketRetrieved(ticket_response) => {
            app.state.jira.add_ticket(ticket_response);
            write_jira_tickets(&app.state.jira.tickets).expect("Failed to persist tickets");
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
            write_jira_tickets(&app.state.jira.tickets).expect("Failed to persist tickets");
        }
        JiraTicketMove(direction) => {
            app.state.jira.swap_tickets(direction);
            write_jira_tickets(&app.state.jira.tickets).expect("Failed to persist tickets");
        }
        _ => {}
    }
}

async fn get_ticket(
    ticket_id: &String,
    config: &JiraConfig,
) -> Result<TicketResponse, Box<dyn Error>> {
    jira_client::get(ticket_id, &config.email, &config.token).await
}