use std::error::Error;
use crate::app::App;
use crate::client::auth_zero_client;
use crate::config::TokenGenerator;
use crate::events::event::AppEvent;
use crate::events::event::AppEvent::{GenerateToken, SetTokenGenFocus, TokenFailed, TokenGenEnvListMove, TokenGenServiceListMove, TokenGenerated};
use crate::utils::update_list_state;

pub fn handle_event(app: &mut App, app_event: AppEvent){
    match (app_event) {
        TokenGenEnvListMove(direction) => {
            let (selected_service, _) =
                app.state.token_generator.get_selected_service_env();

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
            let (service_idx, env_idx) =
                app.state.token_generator.get_selected_service_env();

            app.state
                .token_generator
                .start_token_request(service_idx, env_idx);

            let sender = app.event_sender.clone();
            let config = app.config.tokengenerator.clone();

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
        TokenGenerated(token, service_idx, env_idx) => {
            app.state
                .token_generator
                .set_token_ready(service_idx, env_idx, token);
        }
        TokenFailed(error, service_idx, env_idx) => {
            app.state
                .token_generator
                .set_token_error(service_idx, env_idx, error);
        }
        _ => {}
    }
}

async fn get_token(
    service_idx: usize,
    env_idx: usize,
    config: TokenGenerator,
) -> Result<String, Box<dyn Error>> {
    let service = &config.services[service_idx];
    let credentials = &service.credentials[env_idx];

    auth_zero_client::request_token(
        config.auth0.get_from_env(&credentials.env),
        &credentials.client_id,
        &credentials.client_secret,
        &service.audience,
    )
        .await
}