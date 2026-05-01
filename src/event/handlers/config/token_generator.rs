use crate::app::App;
use crate::event::events::TokenGeneratorConfigEvent;
use crate::event::events::TokenGeneratorConfigEvent::{
    ConfigEdit, ConfigListMove, FormBackspace, FormDelete, FormEnd, FormHome, FormLeft,
    FormNextField, FormPrevField, FormRight, OpenAddService, RemoveService, SubmitConfig,
    SwitchFocus,
};
use crate::event::handlers::config::token_generator::TokenGeneratorConfigEvent::FormChar;
use crate::state::token_generator_config::ActiveEdit;

pub fn handle_event(app: &mut App, event: TokenGeneratorConfigEvent) {
    match event {
        ConfigListMove(direction) => {
            use crate::state::token_generator_config::ConfigFocus;
            let len = app.config.tokengenerator.services.len();
            let editor = &mut app.state.token_generator_config_editor;
            match direction {
                crate::event::events::Direction::Up => {
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
                crate::event::events::Direction::Down => {
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
        FormNextField => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.next(),
            Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.next(),
            None => {}
        },
        FormPrevField => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.prev(),
            Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.prev(),
            None => {}
        },
        FormChar(c) => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_mut().insert(c),
            Some(ActiveEdit::Service(p)) => p.active_field_mut().insert(c),
            None => {}
        },
        FormBackspace => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_mut().backspace(),
            Some(ActiveEdit::Service(p)) => p.active_field_mut().backspace(),
            None => {}
        },
        FormLeft => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_mut().move_left(),
            Some(ActiveEdit::Service(p)) => p.active_field_mut().move_left(),
            None => {}
        },
        FormRight => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_mut().move_right(),
            Some(ActiveEdit::Service(p)) => p.active_field_mut().move_right(),
            None => {}
        },
        FormHome => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_mut().home(),
            Some(ActiveEdit::Service(p)) => p.active_field_mut().home(),
            None => {}
        },
        FormEnd => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_mut().end(),
            Some(ActiveEdit::Service(p)) => p.active_field_mut().end(),
            None => {}
        },
        FormDelete => match &mut app.state.token_generator_config_editor.form {
            Some(ActiveEdit::Auth0(p)) => p.active_field_mut().delete_forward(),
            Some(ActiveEdit::Service(p)) => p.active_field_mut().delete_forward(),
            None => {}
        },
        SubmitConfig => {
            if let Some(form) = app.state.token_generator_config_editor.form.take() {
                match form {
                    ActiveEdit::Auth0(p) => {
                        app.config.tokengenerator.auth0.local = p.local.value().trim().to_string();
                        app.config.tokengenerator.auth0.staging =
                            p.staging.value().trim().to_string();
                        app.config.tokengenerator.auth0.preproduction =
                            p.preprod.value().trim().to_string();
                        app.config.tokengenerator.auth0.production =
                            p.prod.value().trim().to_string();
                        let _ = app.config_loader.write_config(&app.config);
                    }
                    ActiveEdit::Service(p) if p.is_valid() => {
                        let svc = crate::config::model::ServiceConfig {
                            name: p.name.value().trim().to_string(),
                            audience: p.audience.value().trim().to_string(),
                            credentials: p.to_credentials(),
                        };
                        if let Some(idx) = p.edit_index {
                            if let Some(existing) = app.config.tokengenerator.services.get_mut(idx)
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

                    app.config.enforce_feature_invariants();
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
