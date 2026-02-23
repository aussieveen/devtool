use crate::state::jira::Ticket;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Persistence {
    pub jira: Jira,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub(crate) struct Jira {
    pub tickets: Vec<Ticket>,
}
