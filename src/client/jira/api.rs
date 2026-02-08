use crate::client::jira::jira_client;
use crate::client::jira::models::TicketResponse;
use crate::config::JiraConfig;
use crate::events::event::AppEvent::TicketRetrieved;
use crate::events::sender::EventSender;
use std::error::Error;

pub trait JiraApi {
    fn fetch_ticket(&self, ticket_id: String, jira_config: JiraConfig, sender: EventSender);
}

pub struct ImmediateJiraApi {}

impl JiraApi for ImmediateJiraApi {
    fn fetch_ticket(&self, ticket_id: String, jira_config: JiraConfig, sender: EventSender) {
        tokio::spawn(async move {
            match get_ticket(&ticket_id, &jira_config).await {
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

async fn get_ticket(
    ticket_id: &str,
    config: &JiraConfig,
) -> Result<TicketResponse, Box<dyn Error>> {
    jira_client::get(ticket_id, &config.email, &config.token).await
}
