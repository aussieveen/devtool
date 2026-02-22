use crate::app::App;
use crate::error::model::Error;
use crate::events::event::AppEvent;
use crate::events::event::AppEvent::{
    CopyToClipboard, GenerateToken, SetTokenGenFocus, SystemError, TokenFailed,
    TokenGenEnvListMove, TokenGenServiceListMove, TokenGenerated,
};
use crate::state::token_generator::Token;
use crate::utils::string_copy::copy_to_clipboard;
use crate::utils::update_list_state;

pub fn handle_event(app: &mut App, app_event: AppEvent) {
    match app_event {
        TokenGenEnvListMove(direction) => {
            let (selected_service, _) = app.state.token_generator.get_selected_service_env();

            let env_count = app.config.tokengenerator.services[selected_service]
                .credentials
                .len();

            update_list_state::update_list(
                &mut app.state.token_generator.env_list_state,
                direction,
                env_count,
            );
        }
        TokenGenServiceListMove(direction) => {
            let list_state = &mut app.state.token_generator.service_list_state;
            update_list_state::update_list(
                list_state,
                direction,
                app.config.tokengenerator.services.len(),
            );
            app.state.token_generator.env_list_state.select_first();
        }
        SetTokenGenFocus(focus) => {
            app.state.token_generator.focus = focus;
        }
        GenerateToken => {
            let (service_idx, env_idx) = app.state.token_generator.get_selected_service_env();

            app.state.token_generator.start_token_request();

            let sender = app.event_sender.clone();
            let config = app.config.tokengenerator.clone();

            app.auth_zero_api
                .fetch_token(service_idx, env_idx, config, sender);
        }
        TokenGenerated(token, service_idx, env_idx) => {
            app.state
                .token_generator
                .set_token_ready(service_idx, env_idx, token);
        }
        TokenFailed(error, service_idx, env_idx) => {
            app.state
                .token_generator
                .set_token_error(service_idx, env_idx);
            let sender = app.event_sender.clone();
            sender.send(SystemError(Error {
                title: "Error requesting token".to_string(),
                originating_event: "TokenFailed".to_string(),
                tool: "Token Generator".to_string(),
                description: error,
            }));
        }
        CopyToClipboard => {
            let token = app
                .state
                .token_generator
                .get_token_for_selected_service_env();
            if matches!(token, Token::Ready(_))
                && let Some(value) = token.value()
            {
                let result = copy_to_clipboard(value);
                if result.is_err() {
                    let sender = app.event_sender.clone();
                    let description = result.err().unwrap_or_else(|| "Unknown error".to_string());
                    sender.send(SystemError(Error {
                        title: "Failed to copy to clipboard".to_string(),
                        originating_event: "CopyToClipboard".to_string(),
                        tool: "Token Generator".to_string(),
                        description,
                    }));
                }
            }
        }
        _ => {}
    }
}
