use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TicketResponse {
    pub key: String,
    pub fields: Fields,
}

#[derive(Deserialize, Clone)]
pub struct Fields {
    pub assignee: Option<Assignee>,
    pub status: Status,
    pub summary: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Assignee {
    pub display_name: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub name: String,
}
