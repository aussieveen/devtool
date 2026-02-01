use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

pub async fn request_token(
    auth0_url: &String,
    client_id: &String,
    client_secret: &String,
    audience: &String,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut params = HashMap::new();
    params.insert("grant_type", "client_credentials");
    params.insert("client_id", client_id.as_str());
    params.insert("client_secret", client_secret.as_str());
    params.insert("audience", audience.as_str());

    let request = client
        .request(reqwest::Method::POST, auth0_url)
        .form(&params)
        .timeout(Duration::from_secs(3));

    let response = request.send().await?;

    Ok(response.json::<TokenResponse>().await?.access_token)
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
}
