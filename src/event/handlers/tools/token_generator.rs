use crate::app::App;
use crate::event::events::AppEvent::AppLog;
use crate::event::events::GenericEvent::CopyToClipboard;
use crate::event::events::TokenGeneratorEvent::{
    EnvListMove, GenerateToken, ServiceListMove, SetFocus, TokenFailed, TokenGenerated,
};
use crate::event::events::{Event, GenericEvent, TokenGeneratorEvent};
use crate::popup::model::Popup;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::token_generator::Token;
use crate::ui::widgets::popup::{Part, Type};
use crate::utils::string_copy::copy_to_clipboard;
use crate::utils::update_list_state;

const LOG_SOURCE: LogSource = LogSource::TokenGenerator;

pub fn handle_event(app: &mut App, event: TokenGeneratorEvent) {
    match event {
        EnvListMove(direction) => {
            let (selected_service, _) = app.state.token_generator.selected_service_env();

            let env_count = app.config.tokengenerator.services[selected_service]
                .credentials
                .len();

            update_list_state::update_list(
                &mut app.state.token_generator.env_list_state,
                direction,
                env_count,
            );
        }
        ServiceListMove(direction) => {
            let list_state = &mut app.state.token_generator.service_list_state;
            update_list_state::update_list(
                list_state,
                direction,
                app.config.tokengenerator.services.len(),
            );
            app.state.token_generator.env_list_state.select_first();
        }
        SetFocus(focus) => {
            app.state.token_generator.focus = focus;
        }
        GenerateToken => {
            let (service_idx, env_idx) = app.state.token_generator.selected_service_env();

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

            app.event_sender.send_app_event(AppLog(LogEntry::new(
                LogLevel::Info,
                LOG_SOURCE,
                format!("Requesting token: {}/{}", svc_name, env_name),
            )));

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

            app.event_sender.send_app_event(AppLog(LogEntry::new(
                LogLevel::Info,
                LOG_SOURCE,
                format!("Token generated: {}/{}", svc_name, env_name),
            )));

            app.state
                .token_generator
                .set_token_ready(service_idx, env_idx, token);

            app.state.popup = Some(
                Popup::new(
                    Type::Success,
                    "Token Generated".to_string(),
                    vec![Part::Key("c"), Part::Text(" copy to clipboard  ")],
                )
                .with_action('c', "copy", Event::Generic(CopyToClipboard)),
            );
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

            app.state
                .token_generator
                .set_token_error(service_idx, env_idx);

            app.event_sender.send_app_event(AppLog(
                LogEntry::new(
                    LogLevel::Error,
                    LOG_SOURCE,
                    format!("Token request failed — {}/{}", svc_name, env_name),
                )
                .with_detail(error),
            ));
        }
    }
}

pub fn handle_generic_event(app: &mut App, event: GenericEvent) {
    if event == CopyToClipboard {
        let token = app
            .state
            .token_generator
            .token_for_selected_service_env();
        if matches!(token, Token::Ready(_))
            && let Some(value) = token.value()
            && let Err(e) = copy_to_clipboard(value)
        {
            app.event_sender.send_app_event(AppLog(LogEntry::new(
                LogLevel::Warning,
                LOG_SOURCE,
                format!("Copy to clipboard failed: {e}"),
            )));
        }
    }
}
