use crate::config::{Auth0Config, Credentials, TokenGenerator as TokenGeneratorConfig};
use crate::environment::Environment;
use crate::environment::Environment::{Local, Preproduction, Production, Staging};
use crate::events::event::AppEvent;
use crate::events::sender::EventSender;
use crate::state::token_generator::Token::NotGenerated;
use ratatui::widgets::ListState;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub enum Focus {
    Service,
    Env,
}

#[derive(Debug)]
pub(crate) struct TokenGenerator {
    pub auth0config: Auth0Config,
    pub services: Vec<Service>,
    pub env_list_state: ListState,
    pub service_list_state: ListState,
    pub event_sender: EventSender,
    pub focus: Focus,
}

impl TokenGenerator {
    pub(crate) fn new(config: TokenGeneratorConfig, event_sender: EventSender) -> TokenGenerator {
        Self {
            auth0config: config.auth0,
            services: config
                .services
                .into_iter()
                .map(|s| Service {
                    name: s.name,
                    audience: s.audience,
                    credentials: s.credentials,
                    tokens: HashMap::from([
                        (Local, NotGenerated),
                        (Staging, NotGenerated),
                        (Preproduction, NotGenerated),
                        (Production, NotGenerated),
                    ]),
                })
                .collect(),
            env_list_state: ListState::default().with_selected(Some(0)),
            service_list_state: ListState::default().with_selected(Some(0)),
            event_sender: event_sender.clone(),
            focus: Focus::Service,
        }
    }

    pub(crate) async fn set_token(&mut self, service_idx: usize, env_idx: usize) {
        let service = &mut self.services[service_idx];
        let credentials = &service.credentials[env_idx];

        service
            .tokens
            .insert(credentials.env.clone(), Token::Fetching);

        let sender = self.event_sender.clone();

        let auth0_url = self.auth0config.get_from_env(&credentials.env).clone();
        let client_id = credentials.client_id.clone();
        let client_secret = credentials.client_secret.clone();
        let audience = service.audience.clone();

        tokio::spawn(async move {
            let token = match Self::get_token(auth0_url, client_id, client_secret, audience).await {
                Ok(token) => Token::Generated(token),
                Err(err) => Token::Error(err.to_string()),
            };
            sender.send(AppEvent::TokenGenerated(token, service_idx, env_idx))
        });
    }

    async fn get_token(
        auth0_url: String,
        client_id: String,
        client_secret: String,
        audience: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
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

    pub fn get_token_for_selected_service_env(&self) -> String {
        let service_idx = self.service_list_state.selected().unwrap();
        let env_idx = self.env_list_state.selected().unwrap();
        let service = &self.services[service_idx];
        service
            .tokens
            .get(&service.credentials[env_idx].env)
            .unwrap()
            .value()
            .unwrap()
            .to_string()
    }
}

#[derive(Debug)]
pub struct Service {
    pub(crate) name: String,
    audience: String,
    pub(crate) credentials: Vec<Credentials>,
    pub(crate) tokens: HashMap<Environment, Token>,
}

#[derive(Debug)]
pub enum Token {
    NotGenerated,
    Fetching,
    Generated(String),
    Error(String),
}

impl Token {
    pub(crate) fn value(&self) -> Option<&str> {
        match self {
            Token::Generated(s) | Token::Error(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
}
