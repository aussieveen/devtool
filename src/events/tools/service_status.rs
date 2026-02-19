use crate::app::App;
use crate::environment::Environment::{Preproduction, Production, Staging};
use crate::error::model::Error;
use crate::events::event::AppEvent;
use crate::events::event::AppEvent::{
    GetCommitRefErrored, GetCommitRefOk, ScanServiceEnv, ScanServices, ServiceStatusListMove,
    SystemError,
};
use crate::utils::browser::open_link_in_browser;
use crate::utils::string_copy::copy_to_clipboard;
use crate::utils::update_list_state;

pub fn handle_event(app: &mut App, app_event: AppEvent) {
    match app_event {
        ServiceStatusListMove(direction) => {
            let list_state = &mut app.state.service_status.list_state;
            let list_limit = app.state.service_status.services.len();
            update_list_state::update_noneable_list(list_state, direction, list_limit);
        }
        ScanServices => {
            let len = app.state.service_status.services.len();
            let sender = app.event_sender.clone();

            for service_idx in 0..len {
                sender.send(ScanServiceEnv(service_idx, Staging));
                sender.send(ScanServiceEnv(service_idx, Preproduction));
                sender.send(ScanServiceEnv(service_idx, Production));
            }
        }
        ScanServiceEnv(service_idx, env) => {
            app.state
                .service_status
                .set_commit_fetching(service_idx, &env);
            let sender = app.event_sender.clone();
            let config = app.config.servicestatus.clone();

            app.healthcheck_api
                .get_commit_ref(service_idx, env, config, sender);
        }
        GetCommitRefOk(commit, service_idx, env) => {
            app.state
                .service_status
                .set_commit_ok(service_idx, &env, commit);
        }
        GetCommitRefErrored(error, service_idx, env) => {
            app.state
                .service_status
                .set_commit_error(service_idx, &env, error);
        }
        AppEvent::CopyToClipboard => {
            if let Some(link) = get_link_url(app)
                && let Err(e) = copy_to_clipboard(link)
            {
                let sender = app.event_sender.clone();
                sender.send(SystemError(Error {
                    title: "Fail to copy to clipboard".to_string(),
                    originating_event: "CopyToClipboard".to_string(),
                    tool: "Service Status".to_string(),
                    description: e,
                }))
            }
        }
        AppEvent::OpenInBrowser => {
            if let Some(link) = get_link_url(app) {
                open_link_in_browser(link);
            }
        }
        _ => {}
    }
}

fn get_link_url(app: &App) -> Option<String> {
    if !app.state.service_status.has_link() {
        return None;
    }
    if let Some(service_idx) = app.state.service_status.get_selected_service_idx() {
        return app
            .state
            .service_status
            .get_link(&app.config.servicestatus[service_idx].repo);
    }
    None
}
