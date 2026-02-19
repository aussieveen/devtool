use crate::config::model::ServiceConfig;
use crate::state::token_generator::Token::Idle;
use ratatui::widgets::ListState;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
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

    /***
    This function gets the selected service env indexes as it is run as part of a
    synchronous call. The others are all run as part of an asynchronous call meaning
    the values need to be those that were set when the calls started
    */
    pub fn start_token_request(&mut self) {
        let (service_idx, env_idx) = self.get_selected_service_env();
        self.tokens[service_idx][env_idx] = Token::Requesting;
    }

    pub fn set_token_ready(&mut self, service_idx: usize, env_idx: usize, token: String) {
        self.tokens[service_idx][env_idx] = Token::Ready(token);
    }

    pub fn set_token_error(&mut self, service_idx: usize, env_idx: usize) {
        self.tokens[service_idx][env_idx] = Token::Error;
    }

    pub fn get_token_for_selected_service_env(&self) -> &Token {
        &self.tokens[self.get_selected_service()][self.get_selected_env()]
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Idle,
    Requesting,
    Ready(String),
    Error,
}

impl Token {
    pub(crate) fn value(&self) -> Option<&str> {
        match self {
            Token::Ready(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_default_token_generator() -> TokenGenerator {
        TokenGenerator {
            tokens: vec![vec![Idle; 4], vec![Idle; 2]],
            env_list_state: Default::default(),
            service_list_state: Default::default(),
            focus: Focus::Service,
        }
    }

    #[test]
    fn get_selected_service_env_returns_selected() {
        let service_idx = 1;
        let env_idx = 1;
        let mut token_generator = get_default_token_generator();
        token_generator.service_list_state.select(Some(service_idx));
        token_generator.env_list_state.select(Some(env_idx));
        assert_eq!(
            token_generator.get_selected_service_env(),
            (service_idx, env_idx)
        )
    }

    #[test]
    fn get_selected_service_env_returns_default() {
        let mut token_generator = get_default_token_generator();
        token_generator.service_list_state.select(None);
        token_generator.env_list_state.select(None);
        assert_eq!(token_generator.get_selected_service_env(), (0, 0))
    }

    #[test]
    fn start_token_request_sets_token_to_requesting() {
        let mut token_generator = get_default_token_generator();
        token_generator.service_list_state.select(Some(1));
        token_generator.env_list_state.select(Some(1));
        token_generator.start_token_request();
        assert_eq!(
            token_generator.get_token_for_selected_service_env(),
            &Token::Requesting
        );
    }

    #[test]
    fn set_token_ready_sets_token_to_ready() {
        let service_idx = 0;
        let env_idx = 1;
        let token_string = String::from("token");
        let mut token_generator = get_default_token_generator();
        token_generator.set_token_ready(service_idx, env_idx, token_string.clone());

        assert_eq!(
            token_generator.tokens[service_idx][env_idx],
            Token::Ready(token_string)
        );
    }

    #[test]
    fn set_token_error_sets_token_to_error() {
        let service_idx = 0;
        let env_idx = 1;
        let mut token_generator = get_default_token_generator();
        token_generator.set_token_error(service_idx, env_idx);

        assert_eq!(token_generator.tokens[service_idx][env_idx], Token::Error);
    }

    #[test]
    fn get_token_for_selected_service_env_returns_token() {
        let mut token_generator = get_default_token_generator();
        assert_eq!(
            token_generator.get_token_for_selected_service_env(),
            &Token::Idle
        );

        let token_string = String::from("token");
        token_generator.set_token_ready(1, 1, token_string.clone());
        token_generator.service_list_state.select(Some(1));
        token_generator.env_list_state.select(Some(1));
        assert_eq!(
            token_generator.get_token_for_selected_service_env(),
            &Token::Ready(token_string)
        )
    }
}
