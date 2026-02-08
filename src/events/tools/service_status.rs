use crate::app::App;
use crate::client::healthcheck_client;
use crate::config::ServiceStatus;
use crate::environment::Environment;
use crate::environment::Environment::{Preproduction, Production, Staging};
use crate::events::event::AppEvent;
use crate::events::event::AppEvent::{
    GetCommitRefErrored, GetCommitRefOk, ScanServiceEnv, ScanServices, ServiceStatusListMove,
};
use crate::utils::browser::open_link_in_browser;
use crate::utils::string_copy::copy_to_clipboard;
use crate::utils::update_list_state;
use std::error::Error;

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

            tokio::spawn(async move {
                match get_commit_ref(service_idx, &env, config).await {
                    Ok(commit) => {
                        sender.send(GetCommitRefOk(commit, service_idx, env));
                    }
                    Err(err) => {
                        sender.send(GetCommitRefErrored(err.to_string(), service_idx, env));
                    }
                }
            });
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
            if let Some(link) = get_link_url(app) {
                copy_to_clipboard(link).expect("TODO: panic message");
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
        return Some(
            app.state
                .service_status
                .get_link(&app.config.servicestatus[service_idx].repo),
        );
    }
    None
}

async fn get_commit_ref(
    service_idx: usize,
    env: &Environment,
    config: Vec<ServiceStatus>,
) -> Result<String, Box<dyn Error>> {
    let url = format!("{}healthcheck", config[service_idx].get_from_env(env));

    healthcheck_client::get(url).await
}
