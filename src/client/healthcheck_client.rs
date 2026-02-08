use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;

pub async fn get(url: String) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "chrome")
        .header(ACCEPT, "application/json")
        .timeout(Duration::from_secs(3))
        .send()
        .await?;

    Ok(response
        .json::<Healthcheck>()
        .await?
        .version
        .split("_")
        .next()
        .unwrap()
        .to_string())
}

#[derive(Deserialize)]
struct Healthcheck {
    version: String,
}
