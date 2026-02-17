use crate::client::auth_zero::auth_zero_client;
use crate::config::model::TokenGenerator;
use crate::events::event::AppEvent::{TokenFailed, TokenGenerated};
use crate::events::sender::EventSender;
use std::error::Error;

pub trait AuthZeroApi {
    fn fetch_token(
        &self,
        service_idx: usize,
        env_idx: usize,
        config: TokenGenerator,
        sender: EventSender,
    );
}

pub struct ImmediateAuthZeroApi {}

impl AuthZeroApi for ImmediateAuthZeroApi {
    fn fetch_token(
        &self,
        service_idx: usize,
        env_idx: usize,
        config: TokenGenerator,
        sender: EventSender,
    ) {
        tokio::spawn(async move {
            match get_token(service_idx, env_idx, config).await {
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
    service_idx: usize,
    env_idx: usize,
    config: TokenGenerator,
) -> Result<String, Box<dyn Error>> {
    let service = &config.services[service_idx];
    let credentials = &service.credentials[env_idx];

    Ok(auth_zero_client::get_token(
        config.auth0.get_from_env(&credentials.env),
        &credentials.client_id,
        &credentials.client_secret,
        &service.audience,
    )
    .await?
    .access_token)
}
