use serde::Deserialize;
use std::error::Error;

const JIRA_ISSUE_URL: &str = "https://immediateco.atlassian.net/rest/api/3/issue/";

pub async fn get(
    ticket_id: &String,
    username: &String,
    password: &String,
) -> Result<TicketResponse, Box<dyn Error>> {
    let url = format!("{}{}", JIRA_ISSUE_URL, ticket_id.as_str());
    let client = reqwest::Client::builder().build()?;
    let request = client.get(url).basic_auth(username, Some(password));

    Ok(request.send().await?.json::<TicketResponse>().await?)
}

#[derive(Deserialize, Debug)]
pub struct TicketResponse {
    pub key: String,
    pub fields: Fields,
}

#[derive(Deserialize, Debug)]
pub struct Fields {
    pub assignee: Option<Assignee>,
    pub status: Status,
    pub summary: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Assignee {
    pub display_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub name: String,
}
