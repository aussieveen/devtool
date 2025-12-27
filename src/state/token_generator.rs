use std::collections::{BTreeMap, HashMap};
use ratatui::widgets::ListState;
use crate::environment::Environment;
use crate::events::sender::EventSender;
use crate::config::{Credentials, TokenGenerator as TokenGeneratorConfig};
use crate::environment::Environment::{Local, Preproduction, Production, Staging};
use crate::state::token_generator::Token::NoToken;

#[derive(Debug)]
pub enum Focus {
    Service,
    Env
}

#[derive(Debug)]
pub(crate) struct TokenGenerator{
    pub services: Vec<Service>,
    pub env_list_state: ListState,
    pub service_list_state: ListState,
    pub event_sender: EventSender,
    pub focus: Focus
}

impl TokenGenerator {
    pub(crate) fn new(config: TokenGeneratorConfig, event_sender: EventSender) -> TokenGenerator {
        Self{
            services: config.services.into_iter().map(|s| Service{
                name: s.name,
                audience: s.audience,
                credentials: BTreeMap::from(
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
                ),
            }).collect(),
            env_list_state: ListState::default().with_selected(Some(0)),
            service_list_state: ListState::default().with_selected(Some(0)),
            event_sender: event_sender.clone(),
            focus: Focus::Service
        }
    }
}

#[derive(Debug)]
pub struct Service{
    pub(crate) name: String,
    audience: String,
    pub(crate) credentials: BTreeMap<Environment, Option<Credentials>>,
    tokens: HashMap<Environment, Token>
}

#[derive(Debug)]
pub enum Token{
    NoToken,
    Fetching,
    Token(String),
    Error(String)
}

