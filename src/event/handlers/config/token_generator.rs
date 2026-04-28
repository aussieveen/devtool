use crate::event::handlers::config::token_generator::TokenGeneratorConfigEvent::FormChar;
use crate::app::App;
use crate::event::event::TokenGeneratorConfigEvent;
use crate::event::event::TokenGeneratorConfigEvent::{ConfigEdit, ConfigListMove, FormBackspace, FormNextField, FormPrevField, OpenAddService, RemoveService, SubmitConfig, SwitchFocus};
use crate::state::token_generator_config::ActiveEdit;

pub fn handle_event(app: &mut App, event: TokenGeneratorConfigEvent){
    match event {
        ConfigListMove(direction) => {
            use crate::state::token_generator_config::ConfigFocus;
            let len = app.config.tokengenerator.services.len();
            let editor = &mut app.state.token_generator_config_editor;
            match direction {
                crate::event::event::Direction::Up => {
                    if editor.config_focus == ConfigFocus::Services {
                        match editor.table_state.selected() {
                            None | Some(0) => {
                                // Reached the top of services — move back to Auth0
                                editor.config_focus = ConfigFocus::Auth0;
                                editor.table_state.select(None);
                            }
                            _ => editor.table_state.select_previous(),
                        }
                    }
                    // Up while on Auth0 does nothing (already at the top)
                }
                crate::event::event::Direction::Down => {
                    if editor.config_focus == ConfigFocus::Auth0 {
                        if len > 0 {
                            // Drop into the services section
                            editor.config_focus = ConfigFocus::Services;
                            editor.table_state.select(Some(0));
                        }
                    } else if len > 0 {
                        let next = editor.table_state.selected().map(|i| i + 1).unwrap_or(0);
                        editor.table_state.select(Some(next.min(len - 1)));
                    }
                }
            }
        }
        OpenAddService => {
            app.state
                .token_generator_config_editor
                .open_add_service_form();
        }
        FormNextField => {
            match &mut app.state.token_generator_config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.next(),
                Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.next(),
                None => {}
            }
        }
        FormPrevField => {
            match &mut app.state.token_generator_config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.prev(),
                Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.prev(),
                None => {}
            }
        }
        FormChar(c) => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_value_mut().push(c),
            Some(ActiveEdit::Service(p)) => p.active_field_value_mut().push(c),
            None => {}
        },
        FormBackspace => {
            match &mut app.state.token_generator_config_editor.form {
                Some(ActiveEdit::Auth0(p)) => {
                    p.active_field_value_mut().pop();
                }
                Some(ActiveEdit::Service(p)) => {
                    p.active_field_value_mut().pop();
                }
                None => {}
            }
        }
        SubmitConfig => {
            if let Some(form) = app.state.token_generator_config_editor.form.take() {
                match form {
                    ActiveEdit::Auth0(p) => {
                        app.config.tokengenerator.auth0.local = p.local.trim().to_string();
                        app.config.tokengenerator.auth0.staging = p.staging.trim().to_string();
                        app.config.tokengenerator.auth0.preproduction =
                            p.preprod.trim().to_string();
                        app.config.tokengenerator.auth0.production = p.prod.trim().to_string();
                        let _ = app.config_loader.write_config(&app.config);
                    }
                    ActiveEdit::Service(p) if p.is_valid() => {
                        let svc = crate::config::model::ServiceConfig {
                            name: p.name.trim().to_string(),
                            audience: p.audience.trim().to_string(),
                            credentials: p.to_credentials(),
                        };
                        if let Some(idx) = p.edit_index {
                            if let Some(existing) =
                                app.config.tokengenerator.services.get_mut(idx)
                            {
                                *existing = svc;
                            }
                        } else {
                            app.config.tokengenerator.services.push(svc);
                        }
                        app.state.token_generator =
                            crate::state::token_generator::TokenGenerator::new(
                                &app.config.tokengenerator.services,
                            );
                        let _ = app.config_loader.write_config(&app.config);
                    }
                    _ => {} // invalid service form — close without saving
                }
            }
        }
        RemoveService => {
            if let Some(idx) = app
                .state
                .token_generator_config_editor
                .table_state
                .selected()
                && idx < app.config.tokengenerator.services.len()
            {
                app.config.tokengenerator.services.remove(idx);
                app.state.token_generator = crate::state::token_generator::TokenGenerator::new(
                    &app.config.tokengenerator.services,
                );
                let new_len = app.config.tokengenerator.services.len();
                if new_len == 0 {
                    app.state
                        .token_generator_config_editor
                        .table_state
                        .select(None);
                    // Auto-disable the feature since there's no backing config left.
                    app.config.features.token_generator = false;
                    app.state
                        .config_editor
                        .sync_from_features(&app.config.features);
                    app.state.rebuild_tool_list(app.config.jira.is_some());
                } else {
                    app.state
                        .token_generator_config_editor
                        .table_state
                        .select(Some(idx.min(new_len - 1)));
                }
                let _ = app.config_loader.write_config(&app.config);
            }
        }
        ConfigEdit => {
            use crate::state::token_generator_config::ConfigFocus;
            let editor = &app.state.token_generator_config_editor;
            match editor.config_focus {
                ConfigFocus::Auth0 => {
                    let auth0 = app.config.tokengenerator.auth0.clone();
                    app.state
                        .token_generator_config_editor
                        .open_auth0_form(&auth0);
                }
                ConfigFocus::Services => {
                    if let Some(idx) = app
                        .state
                        .token_generator_config_editor
                        .table_state
                        .selected()
                        && let Some(svc) = app.config.tokengenerator.services.get(idx)
                    {
                        let svc = svc.clone();
                        app.state
                            .token_generator_config_editor
                            .open_edit_service_form(idx, &svc);
                    }
                }
            }
        }
        SwitchFocus => {
            use crate::state::token_generator_config::ConfigFocus;
            let editor = &mut app.state.token_generator_config_editor;
            editor.config_focus = match editor.config_focus {
                ConfigFocus::Auth0 => ConfigFocus::Services,
                ConfigFocus::Services => ConfigFocus::Auth0,
            };
            if editor.config_focus == ConfigFocus::Auth0 {
                editor.table_state.select(None);
            } else if !app.config.tokengenerator.services.is_empty() {
                editor.table_state.select(Some(0));
            }
        }
    }
}