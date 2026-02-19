use crate::client::healthcheck::models::Healthcheck;
use reqwest::StatusCode;
use reqwest::header::{ACCEPT, USER_AGENT};
use std::error::Error;
use std::time::Duration;

pub async fn get(base_url: String) -> Result<Healthcheck, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/healthcheck", base_url);

    let request = client
        .get(url)
        .header(USER_AGENT, "chrome")
        .header(ACCEPT, "application/json")
        .timeout(Duration::from_secs(3));

    let response = request.send().await;
    match response {
        Ok(res) => {
            if res.status() == StatusCode::OK {
                Ok(res.json::<Healthcheck>().await?)
            } else if res.status() == StatusCode::SERVICE_UNAVAILABLE {
                Err(format!("{}.", res.status()).into())
            } else {
                Err(format!("{}", res.status()).into())
            }
        }
        Err(e) => {
            if e.is_timeout() {
                Err("Request timed out. VPN connection required"
                    .to_string()
                    .into())
            } else {
                Err(Box::new(e))
            }
        }
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

        let base_url = format!("{}", server.url());
        let result = get(base_url).await;
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

        let base_url = format!("{}", server.url());
        let result = get(base_url).await;

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

        let base_url = format!("{}", server.url());
        let result = get(base_url).await;

        assert!(result.is_err());

        mock.assert_async().await;
    }
}
