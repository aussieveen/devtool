use std::error::Error;
use std::time::Duration;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;

pub async fn get(url: String) -> Result<String, Box<dyn Error>>{
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "chrome")
        .header(ACCEPT, "application/json")
        .timeout(Duration::from_secs(3))
        .send()
        .await?;

    Ok(response.json::<Healthcheck>()
        .await?
        .version
        .split("_")
        .next()
        .unwrap()
        .to_string()
    )
}

#[derive(Deserialize, Debug)]
struct Healthcheck {
    version: String,
}