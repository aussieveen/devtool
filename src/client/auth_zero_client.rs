use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

pub async fn request_token(
    auth0_url: &String,
    client_id: &str,
    client_secret: &str,
    audience: &str,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;

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

    Ok(response.json::<TokenResponse>().await?.access_token)
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}
