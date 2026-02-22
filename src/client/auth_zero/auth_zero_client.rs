use crate::client::auth_zero::models::AuthZeroResponse;
use crate::client::auth_zero::models::AuthZeroResponse::ErrorResponse as AuthZeroErrorResponse;
use crate::client::auth_zero::models::AuthZeroResponse::TokenResponse as AuthZeroTokenResponse;
use crate::client::auth_zero::models::TokenResponse;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use reqwest::Client;

pub async fn get_token(
    client: Client,
    auth0_url: &str,
    client_id: &str,
    client_secret: &str,
    audience: &str,
) -> Result<TokenResponse, Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("grant_type", "client_credentials");
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("audience", audience);

    let request = client
        .request(reqwest::Method::POST, auth0_url)
        .form(&params)
        .timeout(Duration::from_secs(3));

    let response = request.send().await?;

    let body: AuthZeroResponse = serde_json::from_str(response.text().await?.as_str())?;

    match body {
        AuthZeroTokenResponse(r) => Ok(r),
        AuthZeroErrorResponse(e) => {
            Err(format!("Status code: {} - {}", e.error, e.error_description).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_token_returns_token_response() {
        let mut server = mockito::Server::new_async().await;

        let response = serde_json::json!({
            "access_token":"token"
        })
        .to_string();

        let mock = server
            .mock("POST", "/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response)
            .create_async()
            .await;

        let base_url = format!("{}/token", server.url());
        let client = Client::new();
        let result = get_token(client, &base_url, "id", "secret", "audience").await;
        let token_response = result.unwrap();

        assert_eq!(
            token_response,
            TokenResponse {
                access_token: "token".to_string(),
            }
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_token_returns_error_response() {
        let mut server = mockito::Server::new_async().await;

        let response = serde_json::json!({
                "error": 403,
                "error_description": "Access denied"
        })
        .to_string();

        let mock = server
            .mock("POST", "/token")
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(response)
            .create_async()
            .await;

        let base_url = format!("{}/token", server.url());
        let client = Client::new();
        let result = get_token(client, &base_url, "id", "secret", "audience").await;
        let error = result.err();

        assert_eq!(
            error.unwrap().to_string(),
            "Status code: 403 - Access denied"
        );

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_token_handles_generic_error_response() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", "/token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body("FORBIDDEN")
            .create_async()
            .await;

        let base_url = format!("{}/token", server.url());
        let client = Client::new();
        let result = get_token(client, &base_url, "id", "secret", "audience").await;

        assert!(result.is_err());

        mock.assert_async().await;
    }
}
