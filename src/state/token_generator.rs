use std::collections::HashMap;
use ratatui::widgets::ListState;
use crate::environment::Environment;
use crate::events::sender::EventSender;
use crate::config::{Credentials, TokenGenerator as TokenGeneratorConfig};
use crate::environment::Environment::{Local, Preproduction, Production, Staging};
use crate::state::token_generator::Token::NoToken;

#[derive(Debug)]
pub(crate) struct TokenGenerator{
    pub services: Vec<Service>,
    pub env_list_state: ListState,
    pub service_list_state: ListState,
    pub event_sender: EventSender
}

impl TokenGenerator {
    pub(crate) fn new(config: TokenGeneratorConfig, event_sender: EventSender) -> TokenGenerator {
        Self{
            services: config.services.into_iter().map(|s| Service{
                name: s.name,
                audience: s.audience,
                credentials: HashMap::from(
                [
                        (Local, s.local),
                        (Staging, s.staging),
                        (Preproduction, s.preproduction),
                        (Production, s.production)
                    ]
                ),
                tokens: HashMap::from(
                    [
                        (Local, NoToken),
                        (Staging, NoToken),
                        (Preproduction, NoToken),
                        (Production, NoToken)
                    ]
                )
            }).collect(),
            env_list_state: ListState::default().with_selected(Some(0)),
            service_list_state: ListState::default().with_selected(Some(0)),
            event_sender: event_sender.clone()
        }
    }
}

#[derive(Debug)]
pub struct Service{
    name: String,
    audience: String,
    credentials: HashMap<Environment, Credentials>,
    tokens: HashMap<Environment, Token>
}

#[derive(Debug)]
pub enum Token{
    NoToken,
    Fetching,
    Token(String),
    Error(String)
}

