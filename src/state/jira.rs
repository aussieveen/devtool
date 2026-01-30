use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use crate::config::JiraConfig;
use crate::persistence::{read_jira_persistence, write_jira_tickets, Jira as persistence_jira};

#[derive(Debug)]
#[derive(Clone)]
pub struct Jira{
    pub config: JiraConfig,
    pub tickets: Vec<Ticket>,
    pub list_state: ListState,
    pub new_ticket_popup: bool,
    pub new_ticket_id: Option<String>
}

impl Jira {
    const JIRA_URL: &str = "https://immediateco.atlassian.net/rest/api/3/issue/";

    pub fn new(config: JiraConfig) -> Jira{
        Self{
            config,
            tickets: read_jira_persistence().tickets,
            list_state: ListState::default().with_selected(None),
            new_ticket_popup: false,
            new_ticket_id: None
        }
    }

    pub async fn add_ticket(&mut self){
        if let Some(id) = self.new_ticket_id.clone() {
            match Self::get_ticket(id, self.config.email.clone(), self.config.token.clone()).await {
                Ok(ticket) => {
                    self.tickets.push(
                        Ticket::new(
                            ticket.key,
                            ticket.fields.summary,
                            ticket.fields.status.name,
                            match ticket.fields.assignee {
                                Some(assignee) => assignee.display_name,
                                None => "Unassigned".to_string()
                            }
                        )
                    );
                    write_jira_tickets(&self.tickets).expect("Failed to persist tickets");
                },
                Err(e) => {
                    println!("{}", e.to_string());
                    self.tickets.push(
                        Ticket::new(
                            e.to_string(),
                            "??".to_string(),
                            "??".to_string(),
                            "??".to_string()
                        )
                    );

                }
            };
            self.new_ticket_id = None;
        }
    }

    async fn get_ticket(
        id: String,
        username: String,
        password: String
    ) -> Result<TicketResponse, Box<dyn std::error::Error>> {
        let mut url = Self::JIRA_URL.to_owned();
        url.push_str(id.as_str());

        let client = reqwest::Client::builder().build()?;

        let request = client
            .get(url)
            .basic_auth(username, Some(password));

        let response = request.send().await?;

        Ok(response.json::<TicketResponse>().await?)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ticket{
    pub id: String,
    pub title: String,
    pub status: String,
    pub assignee: String,
}

impl Ticket {
    pub fn new(
        id: String,
        title: String,
        status: String,
        assignee: String,
    ) -> Ticket {
        Self{
            id,title,status,assignee
        }
    }
}

#[derive(Deserialize, Debug)]
struct TicketResponse {
    key: String,
    fields: Fields,
}

#[derive(Deserialize, Debug)]
struct Fields {
    assignee: Option<Assignee>,
    status: Status,
    summary: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Assignee {
    display_name: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Status {
    name:String
}
