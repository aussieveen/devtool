use crate::client::jira::jira_client;
use crate::client::jira::models::TicketResponse;
use crate::config::model::JiraConfig;
use crate::error::model::ClientError;
use crate::event::events::AppEvent::AppLog;
use crate::event::events::JiraEvent::TicketRetrieved;
use crate::event::sender::EventSender;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::tools::Tool;
use reqwest::Client;

pub trait JiraApi: Send + Sync {
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

impl Default for ImmediateJiraApi {
    fn default() -> Self {
        Self::new()
    }
}

impl JiraApi for ImmediateJiraApi {
    fn fetch_ticket(&self, ticket_id: String, jira_config: JiraConfig, sender: EventSender) {
        let client = self.client.clone();
        tokio::spawn(async move {
            match ticket(client, &ticket_id, &jira_config).await {
                Ok(ticket) => {
                    sender.send(TicketRetrieved(ticket));
                }
                Err(err) => {
                    sender.send(AppLog(
                        LogEntry::new(
                            LogLevel::Error,
                            LogSource::Tool(Tool::Jira),
                            "Failed to get ticket",
                        )
                        .with_detail(err.to_string()),
                    ));
                }
            }
        });
    }
}

async fn ticket(
    client: Client,
    ticket_id: &str,
    config: &JiraConfig,
) -> Result<TicketResponse, ClientError> {
    jira_client::get(client, &config.url, ticket_id, &config.email, &config.token).await
}
