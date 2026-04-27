use crate::app::App;
use crate::event::event::JiraConfigEvent;
use crate::event::event::JiraConfigEvent::{FormBackspace, FormChar, FormNextField, FormPrevField, OpenEdit, SubmitConfig};

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
                p.active_field_value_mut().push(c);
            }
        }
        FormBackspace => {
            if let Some(p) = &mut app.state.jira_config_editor.form {
                p.active_field_value_mut().pop();
            }
        }
        SubmitConfig => {
            if let Some(form) = app.state.jira_config_editor.form.take() {
                if form.is_empty() {
                    app.config.jira = None;
                } else {
                    app.config.jira = Some(crate::config::model::JiraConfig {
                        url: form.url.trim().to_string(),
                        email: form.email.trim().to_string(),
                        token: form.token.trim().to_string(),
                    });
                }
                let has_jira_config = app.config.jira.is_some();
                app.state.rebuild_tool_list(has_jira_config);
                let _ = app.config_loader.write_config(&app.config);
            }
        }
    }
}