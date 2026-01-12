use ratatui::widgets::ListState;
use crate::config::JiraConfig;

#[derive(Debug)]
pub struct Jira{
    pub config: JiraConfig,
    pub tickets: Vec<Ticket>,
    pub list_state: ListState
}

impl Jira {
    pub fn new(config: JiraConfig) -> Jira{
        Self{
            config,
            tickets: Vec::<Ticket>::new(),
            list_state: ListState::default().with_selected(None)
        }
    }

    pub async fn add_ticket(&mut self, id: String){
        // MAKE REQUEST TO JIRA WITH THAT TICKET ID
        // CREATE TICKET STRUCT
        // ADD TO VEC
    }
}

#[derive(Debug)]
pub struct Ticket{
    pub id: String,
    pub title: String,
    pub status: String,
    pub assignee: String,
    pub description: String
}

impl Ticket {
    pub fn new(
        id: String,
        title: String,
        status: String,
        assignee: String,
        description: String,
    ) -> Ticket {
        Self{
            id,title,status,assignee,description
        }
    }
}