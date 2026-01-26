use ratatui::widgets::ListState;
use serde::Deserialize;
use crate::config::JiraConfig;

#[derive(Debug)]
pub struct Jira{
    pub config: JiraConfig,
    pub tickets: Vec<Ticket>,
    pub list_state: ListState
}

impl Jira {
    const JIRA_URL: &str = "https://immediateco.atlassian.net/rest/api/3/issue/";

    pub fn new(config: JiraConfig) -> Jira{
        Self{
            config,
            tickets: Vec::<Ticket>::new(),
            list_state: ListState::default().with_selected(None)
        }
    }

    pub async fn add_ticket(&mut self, id: String){
        match Self::get_ticket(id, self.config.email.clone(), self.config.token.clone()).await{
            Ok(ticket) => self.tickets.push(
                Ticket::new(
                    ticket.key,
                    ticket.fields.summary,
                    ticket.fields.status.name,
                    ticket.fields.assignee.display_name
                )
            ),
            Err(_) => todo!()
        };
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

#[derive(Debug)]
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
    assignee: Assignee,
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
