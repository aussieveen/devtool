use crate::client::jira::models::JiraResponse::ErrorResponse as JiraErrorResponse;
use crate::client::jira::models::JiraResponse::TicketResponse as JiraTicketResponse;
use crate::client::jira::models::{JiraResponse, TicketResponse};
use crate::error::model::ClientError;
use reqwest::Client;

pub async fn get(
    client: Client,
    base_url: &str,
    ticket_id: &str,
    username: &str,
    password: &str,
) -> Result<TicketResponse, ClientError> {
    let url = format!("{}/issue/{}", base_url, ticket_id);
    let request = client.get(url).basic_auth(username, Some(password));

    let response = request.send().await?;

    let body: JiraResponse = serde_json::from_str(response.text().await?.as_str())?;

    match body {
        JiraTicketResponse(r) => Ok(r),
        JiraErrorResponse(e) => {
            let msg = match e.error_messages.len() {
                0 => "Unknown error".to_string(),
                _ => e.error_messages[0].clone(),
            };
            Err(ClientError::Api(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_returns_deserialized_ticket_response() {
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
            .mock("GET", "/issue/TEST-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ticket_response)
            .create_async()
            .await;

        let username = String::from("user");
        let password = String::from("password");

        let client = Client::new();
        let result = get(client, &server.url(), "TEST-123", &username, &password).await;
        let ticket = result.unwrap();

        assert_eq!(ticket.key, "TEST-123");
        assert_eq!(ticket.fields.assignee.unwrap().display_name, "Tester");
        assert_eq!(ticket.fields.status.name, String::from("In Progress"));
        assert_eq!(ticket.fields.summary, String::from("Testing my code"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_handles_no_assignee() {
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
            .mock("GET", "/issue/TEST-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ticket_response)
            .create_async()
            .await;

        let username = String::from("user");
        let password = String::from("password");

        let client = Client::new();
        let result = get(client, &server.url(), "TEST-123", &username, &password).await;
        let ticket = result.unwrap();

        assert_eq!(ticket.key, "TEST-123");
        assert_eq!(ticket.fields.assignee, None);
        assert_eq!(ticket.fields.status.name, String::from("In Progress"));
        assert_eq!(ticket.fields.summary, String::from("Testing my code"));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_returns_error_response() {
        let mut server = mockito::Server::new_async().await;

        let error = serde_json::json!({
            "errorMessages": [
                "this went wrong",
                "went badly here too"
            ]
        })
        .to_string();

        let mock = server
            .mock("GET", "/issue/TEST-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(error)
            .create_async()
            .await;

        let username = String::from("user");
        let password = String::from("password");

        let client = Client::new();
        let result = get(client, &server.url(), "TEST-123", &username, &password).await;

        assert_eq!(
            result.err().unwrap().to_string(),
            "this went wrong".to_string()
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_returns_unknown_error_response() {
        let mut server = mockito::Server::new_async().await;

        let error = serde_json::json!({
            "errorMessages": []
        })
        .to_string();

        let mock = server
            .mock("GET", "/issue/TEST-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(error)
            .create_async()
            .await;

        let username = String::from("user");
        let password = String::from("password");

        let client = Client::new();
        let result = get(client, &server.url(), "TEST-123", &username, &password).await;

        assert_eq!(
            result.err().unwrap().to_string(),
            "Unknown error".to_string()
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_handles_404_responses() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/issue/TEST-123")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body("Not Found")
            .create_async()
            .await;

        let username = String::from("user");
        let password = String::from("password");

        let client = Client::new();
        let result = get(client, &server.url(), "TEST-123", &username, &password).await;
        assert!(result.is_err());

        mock.assert_async().await;
    }
}
