use crate::app::App;
use crate::event::event::ServiceStatusConfigEvent;
use crate::event::event::ServiceStatusConfigEvent::{
    OpenAddService, OpenEditService, RemoveService, ListMove,
    FormBackspace, FormChar, FormNextField,
    PrevField, SubmitConfig};

pub fn handle_event(app: &mut App, event: ServiceStatusConfigEvent) {
    let editor = &mut app.state.service_status_config_editor;
    match event {
        ListMove(direction) => {
            let len = app.config.servicestatus.len();
            let state = &mut app.state.service_status_config_editor.table_state;
            if len == 0 {
                state.select(None);
            } else {
                match direction {
                    crate::event::event::Direction::Up => match state.selected() {
                        None | Some(0) => state.select(None),
                        _ => state.select_previous(),
                    },
                    crate::event::event::Direction::Down => {
                        let next = state.selected().map(|i| i + 1).unwrap_or(0);
                        state.select(Some(next.min(len - 1)));
                    }
                }
            }
        }
        OpenAddService => {
            editor.open_form();
        }
        OpenEditService => {
            if let Some(idx) = app
                .state
                .service_status_config_editor
                .table_state
                .selected()
                && let Some(svc) = app.config.servicestatus.get(idx)
            {
                app.state
                    .service_status_config_editor
                    .open_edit_form(idx, svc);
            }
        }
        FormNextField => {
            if let Some(form) = &mut editor.form {
                form.active_field = form.active_field.next();
            }
        }
        PrevField => {
            if let Some(form) = &mut editor.form {
                form.active_field = form.active_field.prev();
            }
        }
        FormChar(c) => {
            if let Some(form) = &mut editor.form {
                form.active_field_value_mut().push(c);
            }
        }
        FormBackspace => {
            if let Some(form) = &mut editor.form {
                form.active_field_value_mut().pop();
            }
        }
        SubmitConfig => {
            if let Some(form) = app.state.service_status_config_editor.form.take()
                && form.is_valid()
            {
                let service = crate::config::model::ServiceStatusConfig {
                    name: form.name.trim().to_string(),
                    staging: form.staging.trim().to_string(),
                    preproduction: form.preprod.trim().to_string(),
                    production: form.prod.trim().to_string(),
                    repo: form.repo.trim().to_string(),
                };
                if let Some(idx) = form.edit_index {
                    // Edit existing
                    if let Some(existing) = app.config.servicestatus.get_mut(idx) {
                        *existing = service;
                    }
                } else {
                    // Add new
                    app.config.servicestatus.push(service);
                }
                app.state.service_status = crate::state::service_status::ServiceStatus::new(
                    app.config.servicestatus.len(),
                );
                let _ = app.config_loader.write_config(&app.config);
            }
            // If invalid, just close the form without saving
        }
        RemoveService => {
            if let Some(idx) = app
                .state
                .service_status_config_editor
                .table_state
                .selected()
                && idx < app.config.servicestatus.len()
            {
                app.config.servicestatus.remove(idx);
                app.state.service_status = crate::state::service_status::ServiceStatus::new(
                    app.config.servicestatus.len(),
                );
                // Clamp selection
                let new_len = app.config.servicestatus.len();
                if new_len == 0 {
                    app.state
                        .service_status_config_editor
                        .table_state
                        .select(None);
                    // Auto-disable the feature since there's no backing config left.
                    app.config.features.service_status = false;
                    app.state
                        .config_editor
                        .sync_from_features(&app.config.features);
                    app.state.rebuild_tool_list(app.config.jira.is_some());
                } else {
                    let clamped = idx.min(new_len - 1);
                    app.state
                        .service_status_config_editor
                        .table_state
                        .select(Some(clamped));
                }
                let _ = app.config_loader.write_config(&app.config);
            }
        }
    }
}