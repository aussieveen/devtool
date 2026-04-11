use crate::app::App;
use crate::events::event::AppEvent;
use crate::events::event::AppEvent::{
    AppLog, CopyToClipboard, GenerateToken, SetTokenGenFocus, TokenFailed, TokenGenEnvListMove,
    TokenGenServiceListMove, TokenGenerated,
};
use crate::error::model::Error;
use crate::events::event::AppEvent::SystemError;
use crate::state::log::LogLevel;
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

            let svc_name = app
                .config
                .tokengenerator
                .services
                .get(service_idx)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let env_name = app
                .config
                .tokengenerator
                .services
                .get(service_idx)
                .and_then(|s| s.credentials.get(env_idx))
                .map(|c| c.env.to_string().to_lowercase())
                .unwrap_or_default();

            app.event_sender.send(AppLog(
                LogLevel::Info,
                "token-gen".to_string(),
                format!("Requesting token: {}/{}", svc_name, env_name),
            ));

            app.state.token_generator.start_token_request();

            let sender = app.event_sender.clone();
            let config = app.config.tokengenerator.clone();

            app.auth_zero_api
                .fetch_token(service_idx, env_idx, config, sender);
        }
        TokenGenerated(token, service_idx, env_idx) => {
            let svc_name = app
                .config
                .tokengenerator
                .services
                .get(service_idx)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let env_name = app
                .config
                .tokengenerator
                .services
                .get(service_idx)
                .and_then(|s| s.credentials.get(env_idx))
                .map(|c| c.env.to_string().to_lowercase())
                .unwrap_or_default();

            app.event_sender.send(AppLog(
                LogLevel::Info,
                "token-gen".to_string(),
                format!("Token generated: {}/{}", svc_name, env_name),
            ));

            app.state
                .token_generator
                .set_token_ready(service_idx, env_idx, token);
        }
        TokenFailed(error, service_idx, env_idx) => {
            let svc_name = app
                .config
                .tokengenerator
                .services
                .get(service_idx)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let env_name = app
                .config
                .tokengenerator
                .services
                .get(service_idx)
                .and_then(|s| s.credentials.get(env_idx))
                .map(|c| c.env.to_string().to_lowercase())
                .unwrap_or_default();

            app.event_sender.send(AppLog(
                LogLevel::Error,
                "token-gen".to_string(),
                format!("Token request failed: {}/{} — {}", svc_name, env_name, error),
            ));

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
                && let Err(e) = copy_to_clipboard(value)
            {
                app.event_sender.send(AppLog(
                    LogLevel::Warning,
                    "token-gen".to_string(),
                    format!("Copy to clipboard failed: {}", e),
                ));
            }
        }
        _ => {}
    }
}
