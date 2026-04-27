use crate::app::App;
use crate::environment::Environment::{Preproduction, Production, Staging};
use crate::event::event::AppEvent::{ActivityEvent, AppLog, };
use crate::event::event::{AppEvent, Direction, GenericEvent, ServiceStatusEvent};
use crate::event::event::ServiceStatusEvent::{GetCommitRefErrored, GetCommitRefOk, ScanServiceEnv, Scan, ListMove};
use crate::state::log::LogLevel;
use crate::state::service_status::CommitRefStatus;
use crate::utils::browser::open_link_in_browser;
use crate::utils::string_copy::copy_to_clipboard;

const SERVICE_NAME:&str = "service-status";

pub fn handle_event(app: &mut App, event: ServiceStatusEvent) {
    match event {
        ListMove(direction) => {
            let state = &mut app.state.service_status;
            let len = state.services.len();
            let table_state = &mut state.table_state;

            if len == 0 {
                table_state.select(None);
                return;
            }

            match direction {
                Direction::Up => {
                    if table_state.selected().unwrap_or(0) > 0 {
                        table_state.select_previous();
                    } else {
                        table_state.select(None);
                    }
                }
                Direction::Down => {
                    let selected = table_state.selected().unwrap_or(0);
                    let max = len.saturating_sub(1);
                    if selected < max {
                        table_state.select_next();
                    } else {
                        table_state.select(Some(max));
                    }
                }
            }
        }
        Scan => {
            let len = app.state.service_status.services.len();
            let sender = app.event_sender.clone();

            let service_count = len;
            sender.send_app_event(AppLog(
                LogLevel::Info,
                SERVICE_NAME.to_string(),
                format!("Scan started — {} services × 3 environments", service_count),
            ));

            for service_idx in 0..len {
                sender.send_service_status_event(ScanServiceEnv(service_idx, Staging));
                sender.send_service_status_event(ScanServiceEnv(service_idx, Preproduction));
                sender.send_service_status_event(ScanServiceEnv(service_idx, Production));
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
            let old_status = app.state.service_status.services[service_idx].commit_ref_status();
            app.state
                .service_status
                .set_commit_ok(service_idx, &env, commit);
            let new_status = app.state.service_status.services[service_idx].commit_ref_status();

            if old_status != new_status
                && !matches!(new_status, CommitRefStatus::CommitMissing)
                && let Some(svc_cfg) = app.config.servicestatus.get(service_idx)
            {
                let msg = status_activity_message(&new_status);
                app.event_sender
                    .send_app_event(ActivityEvent(svc_cfg.name.clone(), msg));
            }
        }
        GetCommitRefErrored(error, service_idx, env) => {
            app.state
                .service_status
                .set_commit_error(service_idx, &env, error.clone());

            if let Some(svc_cfg) = app.config.servicestatus.get(service_idx) {
                let source = SERVICE_NAME.to_string();
                let env_label = env.to_string().to_lowercase();
                let message = format!("{}/{}: {}", svc_cfg.name, env_label, friendly_error(&error));
                app.event_sender
                    .send_app_event(AppLog(LogLevel::Error, source, message));
            }
        }
    }
}

pub fn handle_generic_event(app: &mut App, event:GenericEvent){
    match event {
        GenericEvent::CopyToClipboard => {
            if let Some(link) = get_link_url(app)
                && let Err(e) = copy_to_clipboard(link.as_str())
            {
                app.event_sender.send_app_event(AppLog(
                    LogLevel::Warning,
                    SERVICE_NAME.to_string(),
                    format!("Copy to clipboard failed: {}", e),
                ));
            }
        }
        GenericEvent::OpenInBrowser => {
            if let Some(link) = get_link_url(app)
                && let Err(e) = open_link_in_browser(link.as_str())
            {
                app.event_sender.send_app_event(AppLog(
                    LogLevel::Warning,
                    SERVICE_NAME.to_string(),
                    format!("Open in browser failed: {}", e),
                ));
            }
        }
        _ => {}
    }
}

fn get_link_url(app: &App) -> Option<String> {
    if !app.state.service_status.has_link() {
        return None;
    }
    let service_idx = app.state.service_status.get_selected_service_idx()?;
    app.state
        .service_status
        .get_link(&app.config.servicestatus[service_idx].repo)
}

fn status_activity_message(status: &CommitRefStatus) -> String {
    match status {
        CommitRefStatus::AllMatches => "Now in sync across all environments".to_string(),
        CommitRefStatus::StagingPreprodMatch => {
            "Ready for production — staging and preprod match".to_string()
        }
        CommitRefStatus::PreprodProdMatch => "New version in the deployment pipeline".to_string(),
        CommitRefStatus::NothingMatches => "Environments are out of sync".to_string(),
        CommitRefStatus::CommitMissing => {
            "Commit errors detected — may require maintenance".to_string()
        }
    }
}

fn friendly_error(raw: &str) -> String {
    if raw.contains("timed out") {
        "Request timed out — check VPN connection".to_string()
    } else if raw.contains("503") || raw.contains("Service Unavailable") {
        "Service unavailable".to_string()
    } else {
        raw.to_string()
    }
}
