use crate::client::healthcheck::models::Healthcheck;
use crate::error::model::ClientError;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::{Client, StatusCode};
use std::time::Duration;

pub async fn get(client: Client, base_url: &str) -> Result<Healthcheck, ClientError> {
    let url = format!("{}/healthcheck", base_url);

    let response = client
        .get(url)
        .header(USER_AGENT, "chrome")
        .header(ACCEPT, "application/json")
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                ClientError::Api("Request timed out. VPN connection required".to_string())
            } else {
                ClientError::Api(e.to_string())
            }
        })?;

    match response.status() {
        StatusCode::OK => Ok(response.json::<Healthcheck>().await?),
        StatusCode::SERVICE_UNAVAILABLE => Err(ClientError::Api(format!("{}.", response.status()))),
        status => Err(ClientError::Api(format!("{}", status))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_healthcheck_returns_healthcheck_model() {
        let mut server = mockito::Server::new_async().await;

        let response = serde_json::json!({
            "version":"commitref_timestamp"
        })
        .to_string();

        let mock = server
            .mock("GET", "/healthcheck")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response)
            .create_async()
            .await;

        let client = Client::new();
        let result = get(client, server.url().as_str()).await;
        let healthcheck = result.unwrap();

        assert_eq!(
            healthcheck,
            Healthcheck {
                version: "commitref_timestamp".to_string(),
            }
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_healthcheck_handles_service_unavailable() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/healthcheck")
            .with_status(503)
            .with_header("content-type", "application/json")
            .create_async()
            .await;

        let client = Client::new();
        let result = get(client, server.url().as_str()).await;

        assert_eq!(
            result.err().unwrap().to_string(),
            "503 Service Unavailable.".to_string()
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_from_handles_generic_errors() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/healthcheck")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body("Not Found")
            .create_async()
            .await;

        let client = Client::new();
        let result = get(client, server.url().as_str()).await;

        assert!(result.is_err());

        mock.assert_async().await;
    }
}
