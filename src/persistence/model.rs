use crate::tools::jira::state::Ticket;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Persistence {
    pub jira: Jira,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Jira {
    pub tickets: Vec<Ticket>,
}
