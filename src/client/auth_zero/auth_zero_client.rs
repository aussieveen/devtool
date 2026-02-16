use crate::client::auth_zero::models::TokenResponse;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

pub async fn get_token(
    auth0_url: &String,
    client_id: &str,
    client_secret: &str,
    audience: &str,
) -> Result<TokenResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("grant_type", "client_credentials");
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("audience", audience);

    let request = client
        .request(reqwest::Method::POST, auth0_url)
        .form(&params)
        .timeout(Duration::from_secs(3));

    Ok(request.send().await?.json::<TokenResponse>().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_token_returns_token_response() {
        let mut server = mockito::Server::new_async().await;

        let response = serde_json::json!({
            "access_token":"token"
        }).to_string();

        let mock = server
            .mock("POST", "/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response)
            .create_async()
            .await;

        let base_url = format!("{}/token", server.url());
        let result = get_token(
            &base_url,
            "id",
            "secret",
            "audience"
        ).await;
        let token_response = result.unwrap();

        assert_eq!(token_response, TokenResponse{
            access_token: "token".to_string(),
        });

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_token_handles_error_response() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", "/token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body("FORBIDDEN")
            .create_async()
            .await;

        let base_url = format!("{}/token", server.url());
        let result = get_token(
            &base_url,
            "id",
            "secret",
            "audience"
        ).await;

        assert!(result.is_err());

        mock.assert_async().await;
    }
}