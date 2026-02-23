use crate::client::jira::jira_client;
use crate::client::jira::models::TicketResponse;
use crate::config::model::JiraConfig;
use crate::error::model::{ClientError, Error};
use crate::events::event::AppEvent::{SystemError, TicketRetrieved};
use crate::events::sender::EventSender;
use reqwest::Client;

pub trait JiraApi {
    fn fetch_ticket(&self, ticket_id: String, jira_config: JiraConfig, sender: EventSender);
}

pub struct ImmediateJiraApi {
    client: Client,
}

impl ImmediateJiraApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl JiraApi for ImmediateJiraApi {
    fn fetch_ticket(&self, ticket_id: String, jira_config: JiraConfig, sender: EventSender) {
        let client = self.client.clone();
        tokio::spawn(async move {
            match get_ticket(client, &ticket_id, &jira_config).await {
                Ok(ticket) => {
                    sender.send(TicketRetrieved(ticket));
                }
                Err(err) => sender.send(SystemError(Error {
                    title: "Failed to get ticket".to_string(),
                    originating_event: "SubmitTicketId".to_string(),
                    tool: "Jira".to_string(),
                    description: err.to_string(),
                })),
            }
        });
    }
}

async fn get_ticket(
    client: Client,
    ticket_id: &str,
    config: &JiraConfig,
) -> Result<TicketResponse, ClientError> {
    jira_client::get(client, &config.url, ticket_id, &config.email, &config.token).await
}
