use crate::client::jira::models::TicketResponse;
use std::error::Error;

const JIRA_ISSUE_URL: &str = "https://immediateco.atlassian.net/rest/api/3/issue/";

pub async fn get(
    ticket_id: &str,
    username: &String,
    password: &String,
) -> Result<TicketResponse, Box<dyn Error>> {
    get_from(JIRA_ISSUE_URL, ticket_id, username, password).await
}

async fn get_from(
    base_url: &str,
    ticket_id: &str,
    username: &String,
    password: &String,
) -> Result<TicketResponse, Box<dyn Error>> {
    let url = format!("{}{}", base_url, ticket_id);
    let client = reqwest::Client::builder().build()?;
    let request = client.get(url).basic_auth(username, Some(password));

    Ok(request.send().await?.json::<TicketResponse>().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_from_returns_deserialized_ticket_response() {
        let mut server = mockito::Server::new_async().await;

        let ticket_response = serde_json::json!({
            "key":"TEST-123",
            "fields": {
                "assignee" : { "displayName":"Tester" },
                "status" : { "name": "In Progress" },
                "summary" : "Testing my code"
            }
        })
        .to_string();

        let mock = server
            .mock("GET", "/TEST-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ticket_response)
            .create_async()
            .await;

        let base_url = format!("{}/", server.url());
        let username = String::from("user");
        let password = String::from("password");

        let result = get_from(&*base_url, "TEST-123", &username, &password).await;
        let ticket = result.unwrap();

        assert_eq!(ticket.key, "TEST-123");
        assert_eq!(ticket.fields.assignee.unwrap().display_name, "Tester");
        assert_eq!(ticket.fields.status.name, String::from("In Progress"));
        assert_eq!(ticket.fields.summary, String::from("Testing my code"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_from_handles_no_assignee() {
        let mut server = mockito::Server::new_async().await;

        let ticket_response = serde_json::json!({
            "key":"TEST-123",
            "fields": {
                "assignee" : null,
                "status" : { "name": "In Progress" },
                "summary" : "Testing my code"
            }
        })
        .to_string();

        let mock = server
            .mock("GET", "/TEST-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ticket_response)
            .create_async()
            .await;

        let base_url = format!("{}/", server.url());
        let username = String::from("user");
        let password = String::from("password");

        let result = get_from(&*base_url, "TEST-123", &username, &password).await;
        let ticket = result.unwrap();

        assert_eq!(ticket.key, "TEST-123");
        assert_eq!(ticket.fields.assignee, None);
        assert_eq!(ticket.fields.status.name, String::from("In Progress"));
        assert_eq!(ticket.fields.summary, String::from("Testing my code"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_from_handles_404_responses() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/TEST-123")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body("Not Found")
            .create_async()
            .await;

        let base_url = format!("{}/", server.url());
        let username = String::from("user");
        let password = String::from("password");

        let result = get_from(&*base_url, "TEST-123", &username, &password).await;
        assert!(result.is_err());

        mock.assert_async().await;
    }
}
