use crate::config::ServiceConfig;
use crate::state::token_generator::Token::Idle;
use ratatui::widgets::ListState;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub enum Focus {
    Service,
    Env,
}

pub(crate) struct TokenGenerator {
    pub tokens: Vec<Vec<Token>>,
    pub env_list_state: ListState,
    pub service_list_state: ListState,
    pub focus: Focus,
}

impl TokenGenerator {
    pub(crate) fn new(services: &[ServiceConfig]) -> TokenGenerator {
        let tokens = services
            .iter()
            .map(|s| vec![Idle; s.credentials.len()])
            .collect();

        Self {
            tokens,
            env_list_state: ListState::default().with_selected(Some(0)),
            service_list_state: ListState::default().with_selected(Some(0)),
            focus: Focus::Service,
        }
    }

    pub fn get_selected_service_env(&self) -> (usize, usize) {
        (self.get_selected_service(), self.get_selected_env())
    }

    fn get_selected_service(&self) -> usize {
        self.service_list_state.selected().unwrap_or_default()
    }

    fn get_selected_env(&self) -> usize {
        self.env_list_state.selected().unwrap_or_default()
    }

    pub fn start_token_request(&mut self, service_idx: usize, env_idx: usize) {
        self.tokens[service_idx][env_idx] = Token::Requesting;
    }

    pub fn set_token_ready(&mut self, service_idx: usize, env_idx: usize, token: String) {
        self.tokens[service_idx][env_idx] = Token::Ready(token);
    }

    pub fn set_token_error(&mut self, service_idx: usize, env_idx: usize, error: String) {
        self.tokens[service_idx][env_idx] = Token::Error(error);
    }

    pub fn get_token_for_selected_service_env(&self) -> &Token {
        &self.tokens[self.get_selected_service()][self.get_selected_env()]
    }
}

#[derive(Clone, PartialEq)]
pub enum Token {
    Idle,
    Requesting,
    Ready(String),
    Error(String),
}

impl Token {
    pub(crate) fn value(&self) -> Option<&str> {
        match self {
            Token::Ready(s) | Token::Error(s) => Some(s.as_str()),
            _ => None,
        }
    }
}
