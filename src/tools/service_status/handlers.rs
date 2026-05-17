use super::ServiceStatusPlugin;
use super::state::CommitRefStatus;
use crate::environment::Environment::{Preproduction, Production, Staging};
use crate::event::events::AppEvent::{ActivityEvent, AppLog, RebuildToolList};
use crate::event::events::ServiceStatusConfigEvent::{
    FormBackspace, FormDelete, FormEnd, FormHome, FormLeft, FormNextField, FormRight,
    ListMove as ConfigListMove, OpenAddService, OpenEditService, PrevField, RemoveService,
    SubmitConfig,
};
use crate::event::events::ServiceStatusEvent::{
    GetCommitRefErrored, GetCommitRefOk, ListMove, Scan, ScanServiceEnv,
};
use crate::event::events::{ServiceStatusConfigEvent, ServiceStatusEvent};
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::tools::context::PluginContext;

const LOG_SOURCE: LogSource = LogSource::ServiceStatus;

impl ServiceStatusPlugin {
    pub(super) fn handle_tool_event(&mut self, event: ServiceStatusEvent, ctx: &mut PluginContext) {
        match event {
            ListMove(direction) => {
                let len = self.state.services.len();
                let table_state = &mut self.state.table_state;

                if len == 0 {
                    table_state.select(None);
                    return;
                }

                match direction {
                    crate::event::events::Direction::Up => {
                        if table_state.selected().unwrap_or(0) > 0 {
                            table_state.select_previous();
                        } else {
                            table_state.select(None);
                        }
                    }
                    crate::event::events::Direction::Down => {
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
                let len = self.state.services.len();
                let sender = ctx.sender.clone();
                sender.send_app_event(AppLog(LogEntry::new(
                    LogLevel::Info,
                    LOG_SOURCE,
                    format!("Scan started — {} services × 3 environments", len),
                )));
                for service_idx in 0..len {
                    sender.send_service_status_event(ScanServiceEnv(service_idx, Staging));
                    sender.send_service_status_event(ScanServiceEnv(service_idx, Preproduction));
                    sender.send_service_status_event(ScanServiceEnv(service_idx, Production));
                }
            }
            ScanServiceEnv(service_idx, env) => {
                self.state.set_commit_fetching(service_idx, &env);
                let sender = ctx.sender.clone();
                let config = ctx.config.servicestatus.clone();
                self.healthcheck_api.commit_ref(service_idx, env, config.into(), sender);
            }
            GetCommitRefOk(commit, service_idx, env) => {
                let old_status = self.state.services[service_idx].commit_ref_status();
                self.state.set_commit_ok(service_idx, &env, commit);
                let new_status = self.state.services[service_idx].commit_ref_status();

                if old_status != new_status
                    && !matches!(new_status, CommitRefStatus::CommitMissing)
                    && let Some(svc_cfg) = ctx.config.servicestatus.get(service_idx)
                {
                    let msg = status_activity_message(&new_status);
                    ctx.sender.send_app_event(ActivityEvent(svc_cfg.name.clone(), msg));
                }
            }
            GetCommitRefErrored(error, service_idx, env) => {
                self.state.set_commit_error(service_idx, &env, error.clone());
                if let Some(svc_cfg) = ctx.config.servicestatus.get(service_idx) {
                    let env_label = env.to_string().to_lowercase();
                    ctx.sender.send_app_event(AppLog(LogEntry::new(
                        LogLevel::Warning,
                        LOG_SOURCE,
                        format!("{}/{}: {}", svc_cfg.name, env_label, friendly_error(&error)),
                    )));
                }
            }
        }
    }

    pub(super) fn handle_config_event(&mut self, event: ServiceStatusConfigEvent, ctx: &mut PluginContext) {
        match event {
            ConfigListMove(direction) => {
                let len = ctx.config.servicestatus.len();
                let state = &mut self.config_editor.table_state;
                if len == 0 {
                    state.select(None);
                } else {
                    match direction {
                        crate::event::events::Direction::Up => match state.selected() {
                            None | Some(0) => state.select(None),
                            _ => state.select_previous(),
                        },
                        crate::event::events::Direction::Down => {
                            let next = state.selected().map(|i| i + 1).unwrap_or(0);
                            state.select(Some(next.min(len - 1)));
                        }
                    }
                }
            }
            OpenAddService => {
                self.config_editor.open_form();
            }
            OpenEditService => {
                if let Some(idx) = self.config_editor.table_state.selected()
                    && let Some(svc) = ctx.config.servicestatus.get(idx)
                {
                    let svc = svc.clone();
                    self.config_editor.open_edit_form(idx, &svc);
                }
            }
            FormNextField => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field = form.active_field.next();
                }
            }
            PrevField => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field = form.active_field.prev();
                }
            }
            crate::event::events::ServiceStatusConfigEvent::FormChar(c) => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().insert(c);
                }
            }
            FormBackspace => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().backspace();
                }
            }
            FormLeft => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().move_left();
                }
            }
            FormRight => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().move_right();
                }
            }
            FormHome => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().home();
                }
            }
            FormEnd => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().end();
                }
            }
            FormDelete => {
                if let Some(form) = &mut self.config_editor.form {
                    form.active_field_mut().delete_forward();
                }
            }
            SubmitConfig => {
                if let Some(form) = self.config_editor.form.take()
                    && form.is_valid()
                {
                    let service = crate::config::model::ServiceStatusConfig {
                        name: form.name.value().trim().to_string(),
                        staging: form.staging.value().trim().to_string(),
                        preproduction: form.preprod.value().trim().to_string(),
                        production: form.prod.value().trim().to_string(),
                        repo: form.repo.value().trim().to_string(),
                    };
                    if let Some(idx) = form.edit_index {
                        if let Some(existing) = ctx.config.servicestatus.get_mut(idx) {
                            *existing = service;
                        }
                    } else {
                        ctx.config.servicestatus.push(service);
                    }
                    self.state = super::state::ServiceStatus::new(ctx.config.servicestatus.len());
                    let _ = ctx.config_loader.write_config(ctx.config);
                }
            }
            RemoveService => {
                if let Some(idx) = self.config_editor.table_state.selected()
                    && idx < ctx.config.servicestatus.len()
                {
                    ctx.config.servicestatus.remove(idx);
                    self.state = super::state::ServiceStatus::new(ctx.config.servicestatus.len());
                    let new_len = ctx.config.servicestatus.len();
                    if new_len == 0 {
                        self.config_editor.table_state.select(None);
                        ctx.config.enforce_feature_invariants();
                        ctx.sender.send_app_event(RebuildToolList);
                    } else {
                        let clamped = idx.min(new_len - 1);
                        self.config_editor.table_state.select(Some(clamped));
                    }
                    let _ = ctx.config_loader.write_config(ctx.config);
                }
            }
        }
    }
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
