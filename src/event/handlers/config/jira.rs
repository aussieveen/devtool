use crate::app::App;
use crate::event::events::JiraConfigEvent;
use crate::event::events::JiraConfigEvent::{FormBackspace, FormChar, FormDelete, FormEnd, FormHome, FormLeft, FormNextField, FormPrevField, FormRight, OpenEdit, SubmitConfig};

pub fn handle_event(app: &mut App, event: JiraConfigEvent){
    match event {
        OpenEdit => {
            app.state
                .jira_config_editor
                .open_form(app.config.jira.as_ref());
        }
        FormNextField => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field = p.active_field.next();
            }
        }
        FormPrevField => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field = p.active_field.prev();
            }
        }
        FormChar(c) => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_mut().insert(c);
            }
        }
        FormBackspace => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_mut().backspace();
            }
        }
        FormLeft => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_mut().move_left();
            }
        }
        FormRight => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_mut().move_right();
            }
        }
        FormHome => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_mut().home();
            }
        }
        FormEnd => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_mut().end();
            }
        }
        FormDelete => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_mut().delete_forward();
            }
        }
        SubmitConfig => {
            if let Some(form) = app.state.jira_config_editor.form.take() {
                if form.is_empty() {
                    app.config.jira = None;
                } else {
                    app.config.jira = Some(crate::config::model::JiraConfig {
                        url: form.url.value().trim().to_string(),
                        email: form.email.value().trim().to_string(),
                        token: form.token.value().trim().to_string(),
                    });
                }
                let has_jira_config = app.config.jira.is_some();
                app.state.rebuild_tool_list(has_jira_config);
                let _ = app.config_loader.write_config(&app.config);
            }
        }
    }
}