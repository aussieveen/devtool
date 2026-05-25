use super::TokenGeneratorPlugin;
use super::state::TokenGenerator;
use crate::event::events::AppEvent::AppLog;
use crate::event::events::AppEvent::RebuildToolList;
use crate::event::events::GenericEvent::CopyToClipboard;
use crate::event::events::TokenGeneratorConfigEvent::{
    ConfigEdit, ConfigListMove, FormBackspace, FormDelete, FormEnd, FormHome, FormLeft,
    FormNextField, FormPrevField, FormRight, OpenAddService, RemoveService, SubmitConfig,
};
use crate::event::events::TokenGeneratorEvent::{
    EnvListMove, GenerateToken, ServiceListMove, SetFocus, TokenFailed, TokenGenerated,
};
use crate::event::events::{Event, TokenGeneratorConfigEvent, TokenGeneratorEvent};
use crate::popup::model::Popup;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;
use crate::ui::widgets::popup::{Part, Type};
use crate::utils::update_list_state;

const LOG_SOURCE: LogSource = LogSource::Tool(Tool::TokenGenerator);

impl TokenGeneratorPlugin {
    pub(super) fn handle_tool_event(
        &mut self,
        event: TokenGeneratorEvent,
        ctx: &mut PluginContext,
    ) {
        match event {
            EnvListMove(direction) => {
                let (selected_service, _) = self.state.selected_service_env();
                let env_count = ctx.config.tokengenerator.services[selected_service]
                    .credentials
                    .len();
                update_list_state::update_list(
                    &mut self.state.env_list_state,
                    direction,
                    env_count,
                );
            }
            ServiceListMove(direction) => {
                update_list_state::update_list(
                    &mut self.state.service_list_state,
                    direction,
                    ctx.config.tokengenerator.services.len(),
                );
                self.state.env_list_state.select_first();
            }
            SetFocus(focus) => {
                self.state.focus = focus;
            }
            GenerateToken => {
                let (service_idx, env_idx) = self.state.selected_service_env();
                let svc_name = ctx
                    .config
                    .tokengenerator
                    .services
                    .get(service_idx)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let env_name = ctx
                    .config
                    .tokengenerator
                    .services
                    .get(service_idx)
                    .and_then(|s| s.credentials.get(env_idx))
                    .map(|c| c.env.to_string().to_lowercase())
                    .unwrap_or_default();

                ctx.sender.send(AppLog(LogEntry::new(
                    LogLevel::Info,
                    LOG_SOURCE,
                    format!("Requesting token: {}/{}", svc_name, env_name),
                )));

                self.state.start_token_request();

                let sender = ctx.sender.clone();
                let config = ctx.config.tokengenerator.clone();
                self.auth_zero_api
                    .fetch_token(service_idx, env_idx, config, sender);
            }
            TokenGenerated(token, service_idx, env_idx) => {
                let svc_name = ctx
                    .config
                    .tokengenerator
                    .services
                    .get(service_idx)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let env_name = ctx
                    .config
                    .tokengenerator
                    .services
                    .get(service_idx)
                    .and_then(|s| s.credentials.get(env_idx))
                    .map(|c| c.env.to_string().to_lowercase())
                    .unwrap_or_default();

                ctx.sender.send(AppLog(LogEntry::new(
                    LogLevel::Info,
                    LOG_SOURCE,
                    format!("Token generated: {}/{}", svc_name, env_name),
                )));

                self.state.set_token_ready(service_idx, env_idx, token);

                *ctx.popup = Some(
                    Popup::new(
                        Type::Success,
                        "Token Generated".to_string(),
                        vec![Part::Key("c"), Part::Text(" copy to clipboard  ")],
                    )
                    .with_action('c', "copy", Event::Generic(CopyToClipboard)),
                );
            }
            TokenFailed(error, service_idx, env_idx) => {
                let svc_name = ctx
                    .config
                    .tokengenerator
                    .services
                    .get(service_idx)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let env_name = ctx
                    .config
                    .tokengenerator
                    .services
                    .get(service_idx)
                    .and_then(|s| s.credentials.get(env_idx))
                    .map(|c| c.env.to_string().to_lowercase())
                    .unwrap_or_default();

                self.state.set_token_error(service_idx, env_idx);

                ctx.sender.send(AppLog(
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

    pub(super) fn handle_config_event(
        &mut self,
        event: TokenGeneratorConfigEvent,
        ctx: &mut PluginContext,
    ) {
        use super::config_editor::ActiveEdit;
        match event {
            ConfigListMove(direction) => {
                // Unified list: index 0 = Auth0, indices 1+ = services.
                let total = 1 + ctx.config.tokengenerator.services.len();
                let editor = &mut self.config_editor;
                match direction {
                    crate::event::events::Direction::Up => {
                        let current = editor.table_state.selected().unwrap_or(0);
                        editor.table_state.select(Some(current.saturating_sub(1)));
                    }
                    crate::event::events::Direction::Down => {
                        let current = editor.table_state.selected().unwrap_or(0);
                        editor
                            .table_state
                            .select(Some((current + 1).min(total - 1)));
                    }
                }
            }
            OpenAddService => {
                self.config_editor.open_add_service_form();
            }
            FormNextField => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.next(),
                Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.next(),
                None => {}
            },
            FormPrevField => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.prev(),
                Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.prev(),
                None => {}
            },
            crate::event::events::TokenGeneratorConfigEvent::FormChar(c) => {
                match &mut self.config_editor.form {
                    Some(ActiveEdit::Auth0(p)) => p.active_field_mut().insert(c),
                    Some(ActiveEdit::Service(p)) => p.active_field_mut().insert(c),
                    None => {}
                }
            }
            FormBackspace => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field_mut().backspace(),
                Some(ActiveEdit::Service(p)) => p.active_field_mut().backspace(),
                None => {}
            },
            FormLeft => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field_mut().move_left(),
                Some(ActiveEdit::Service(p)) => p.active_field_mut().move_left(),
                None => {}
            },
            FormRight => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field_mut().move_right(),
                Some(ActiveEdit::Service(p)) => p.active_field_mut().move_right(),
                None => {}
            },
            FormHome => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field_mut().home(),
                Some(ActiveEdit::Service(p)) => p.active_field_mut().home(),
                None => {}
            },
            FormEnd => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field_mut().end(),
                Some(ActiveEdit::Service(p)) => p.active_field_mut().end(),
                None => {}
            },
            FormDelete => match &mut self.config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field_mut().delete_forward(),
                Some(ActiveEdit::Service(p)) => p.active_field_mut().delete_forward(),
                None => {}
            },
            SubmitConfig => {
                if let Some(form) = self.config_editor.form.take() {
                    match form {
                        ActiveEdit::Auth0(p) => {
                            ctx.config.tokengenerator.auth0.local =
                                p.local.value().trim().to_string();
                            ctx.config.tokengenerator.auth0.staging =
                                p.staging.value().trim().to_string();
                            ctx.config.tokengenerator.auth0.preproduction =
                                p.preprod.value().trim().to_string();
                            ctx.config.tokengenerator.auth0.production =
                                p.prod.value().trim().to_string();
                            let _ = ctx.config_loader.write_config(ctx.config);
                        }
                        ActiveEdit::Service(p) if p.is_valid() => {
                            let svc = crate::config::model::ServiceConfig {
                                name: p.name.value().trim().to_string(),
                                audience: p.audience.value().trim().to_string(),
                                credentials: p.to_credentials(),
                            };
                            if let Some(idx) = p.edit_index {
                                if let Some(existing) =
                                    ctx.config.tokengenerator.services.get_mut(idx)
                                {
                                    *existing = svc;
                                }
                            } else {
                                ctx.config.tokengenerator.services.push(svc);
                            }
                            self.state = TokenGenerator::new(&ctx.config.tokengenerator.services);
                            let _ = ctx.config_loader.write_config(ctx.config);
                        }
                        _ => {}
                    }
                }
            }
            RemoveService => {
                if let Some(svc_idx) = self.config_editor.selected_service_idx()
                    && svc_idx < ctx.config.tokengenerator.services.len()
                {
                    ctx.config.tokengenerator.services.remove(svc_idx);
                    self.state = TokenGenerator::new(&ctx.config.tokengenerator.services);
                    let new_len = ctx.config.tokengenerator.services.len();
                    // After removal, stay on the same list position (clamped), falling back to Auth0 row.
                    let new_list_idx = if new_len == 0 {
                        ctx.sender.send(RebuildToolList);
                        0 // back to Auth0 row
                    } else {
                        // Keep list index pointing at same position (1 + svc_idx, clamped).
                        (1 + svc_idx).min(new_len) // new_len = 1-based last service index
                    };
                    self.config_editor.table_state.select(Some(new_list_idx));
                    let _ = ctx.config_loader.write_config(ctx.config);
                }
            }
            ConfigEdit => {
                if self.config_editor.is_auth0_selected() {
                    let auth0 = ctx.config.tokengenerator.auth0.clone();
                    self.config_editor.open_auth0_form(&auth0);
                } else if let Some(svc_idx) = self.config_editor.selected_service_idx()
                    && let Some(svc) = ctx.config.tokengenerator.services.get(svc_idx)
                {
                    let svc = svc.clone();
                    self.config_editor.open_edit_service_form(svc_idx, &svc);
                }
            }
        }
    }
}
