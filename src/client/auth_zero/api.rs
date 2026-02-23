use crate::client::auth_zero::auth_zero_client;
use crate::config::model::TokenGenerator;
use crate::error::model::ClientError;
use crate::events::event::AppEvent::{TokenFailed, TokenGenerated};
use crate::events::sender::EventSender;
use reqwest::Client;

pub trait AuthZeroApi {
    fn fetch_token(
        &self,
        service_idx: usize,
        env_idx: usize,
        config: TokenGenerator,
        sender: EventSender,
    );
}

pub struct ImmediateAuthZeroApi {
    client: Client,
}

impl ImmediateAuthZeroApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl AuthZeroApi for ImmediateAuthZeroApi {
    fn fetch_token(
        &self,
        service_idx: usize,
        env_idx: usize,
        config: TokenGenerator,
        sender: EventSender,
    ) {
        let client = self.client.clone();
        tokio::spawn(async move {
            match get_token(client, service_idx, env_idx, config).await {
                Ok(token) => {
                    sender.send(TokenGenerated(token, service_idx, env_idx));
                }
                Err(err) => {
                    sender.send(TokenFailed(err.to_string(), service_idx, env_idx));
                }
            }
        });
    }
}

async fn get_token(
    client: Client,
    service_idx: usize,
    env_idx: usize,
    config: TokenGenerator,
) -> Result<String, ClientError> {
    let service = &config.services[service_idx];
    let credentials = &service.credentials[env_idx];

    Ok(auth_zero_client::get_token(
        client,
        config.auth0.get_from_env(&credentials.env),
        &credentials.client_id,
        &credentials.client_secret,
        &service.audience,
    )
    .await?
    .access_token)
}
