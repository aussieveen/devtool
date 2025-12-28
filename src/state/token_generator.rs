use std::collections::{BTreeMap, HashMap};
use ratatui::widgets::ListState;
use crate::environment::Environment;
use crate::events::sender::EventSender;
use crate::config::{Auth0Config, Credentials, TokenGenerator as TokenGeneratorConfig};
use crate::environment::Environment::{Local, Preproduction, Production, Staging};
use crate::events::event::Event::App;
use crate::state::token_generator::Token::NoToken;

#[derive(Debug)]
pub enum Focus {
    Service,
    Env
}

#[derive(Debug)]
pub(crate) struct TokenGenerator{
    pub auth0config: Auth0Config,
    pub services: Vec<Service>,
    pub env_list_state: ListState,
    pub service_list_state: ListState,
    pub event_sender: EventSender,
    pub focus: Focus
}

impl TokenGenerator {
    pub(crate) fn new(config: TokenGeneratorConfig, event_sender: EventSender) -> TokenGenerator {
        Self{
            auth0config: config.auth0,
            services: config.services.into_iter().map(|s| Service{
                name: s.name,
                audience: s.audience,
                credentials: s.credentials,
                tokens: HashMap::from(
                    [
                        (Local, NoToken),
                        (Staging, NoToken),
                        (Preproduction, NoToken),
                        (Production, NoToken)
                    ]
                ),
            }).collect(),
            env_list_state: ListState::default().with_selected(Some(0)),
            service_list_state: ListState::default().with_selected(Some(0)),
            event_sender: event_sender.clone(),
            focus: Focus::Service
        }
    }

    pub(crate) async fn set_token(&self, service_idx: usize, env_idx: usize) {
        let service = &self.services[service_idx];
        let credentials = &service.credentials[env_idx];

        let sender = self.event_sender.clone();

        let auth0_url = self.auth0config.get_from_env(&credentials.env).clone();
        let client_id = credentials.client_id.clone();
        let client_secret = credentials.client_secret.clone();
        let audience = service.audience.clone();

        tokio::spawn(async move {
            let token = match Self::get_token(auth0_url, client_id , client_secret , audience ).await {
                Ok(token) => Token::Token(token),
                Err(err) => {
                    Token::Error(err.to_string())
                }
            };
            // sender.send(AppEvent::TokenGenerated(service_idx, env_idx, token))
        });
    }

    async fn get_token(auth0_url: String, client_id: String, client_secret: String, audience: String) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .build()?;

        let mut params = HashMap::new();
        params.insert("grant_type", "client_credentials");
        params.insert("client_id", client_id.as_str());
        params.insert("client_secret", client_secret.as_str());
        params.insert("audience", audience.as_str());

        let request = client.request(reqwest::Method::POST, auth0_url)
            .form(&params);

        let response = request.send().await?;
        let body = response.text().await?;

        println!("{}", body);

        Ok(body)
    }
}

#[derive(Debug)]
pub struct Service{
    pub(crate) name: String,
    audience: String,
    pub(crate) credentials: Vec<Credentials>,
    tokens: HashMap<Environment, Token>
}

#[derive(Debug)]
pub enum Token{
    NoToken,
    Fetching,
    Token(String),
    Error(String)
}

