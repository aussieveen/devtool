use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::prelude::Rect;

use crate::client::auth_zero::api::AuthZeroApi;
use crate::config::model::{Config, Features};
use crate::event::events::AppEvent::AppLog;
use crate::event::events::AppEvent::RebuildToolList;
use crate::event::events::GenericEvent::CopyToClipboard;
use crate::event::events::TokenGeneratorConfigEvent::{
    ConfigEdit, ConfigListMove, FormBackspace, FormDelete, FormEnd, FormHome, FormLeft,
    FormNextField, FormPrevField, FormRight, OpenAddService, RemoveService, SubmitConfig,
    SwitchFocus,
};
use crate::event::events::TokenGeneratorEvent::{
    EnvListMove, GenerateToken, ServiceListMove, SetFocus, TokenFailed, TokenGenerated,
};
use crate::event::events::{Event, GenericEvent, TokenGeneratorConfigEvent, TokenGeneratorEvent};
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{
    Editing, TokenGen as TokenGenCtx, Tool as ToolCtx, ToolConfig,
};
use crate::input::key_event_map::KeyEventMap;
use crate::popup::model::Popup;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::token_generator::{Focus, Token, TokenGenerator};
use crate::state::token_generator_config::{ActiveEdit, TokenGeneratorConfigEditor};
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;
use crate::tools::plugin::Plugin;
use crate::ui::widgets::config::token_generator as config_widget;
use crate::ui::widgets::popup::{Part, Type};
use crate::ui::widgets::tools::token_generator as widget;
use crate::utils::string_copy::copy_to_clipboard;
use crate::utils::update_list_state;

const LOG_SOURCE: LogSource = LogSource::TokenGenerator;

pub struct TokenGeneratorPlugin {
    state:         TokenGenerator,
    config_editor: TokenGeneratorConfigEditor,
    auth_zero_api: Arc<dyn AuthZeroApi>,
}

impl TokenGeneratorPlugin {
    pub fn new(config: &Config, auth_zero_api: Arc<dyn AuthZeroApi>) -> Self {
        Self {
            state:         TokenGenerator::new(&config.tokengenerator.services),
            config_editor: TokenGeneratorConfigEditor::new(),
            auth_zero_api,
        }
    }

    fn handle_tool_event(&mut self, event: TokenGeneratorEvent, ctx: &mut PluginContext) {
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
                let svc_name = ctx.config.tokengenerator.services
                    .get(service_idx)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let env_name = ctx.config.tokengenerator.services
                    .get(service_idx)
                    .and_then(|s| s.credentials.get(env_idx))
                    .map(|c| c.env.to_string().to_lowercase())
                    .unwrap_or_default();

                ctx.sender.send_app_event(AppLog(LogEntry::new(
                    LogLevel::Info,
                    LOG_SOURCE,
                    format!("Requesting token: {}/{}", svc_name, env_name),
                )));

                self.state.start_token_request();

                let sender = ctx.sender.clone();
                let config = ctx.config.tokengenerator.clone();
                self.auth_zero_api.fetch_token(service_idx, env_idx, config, sender);
            }
            TokenGenerated(token, service_idx, env_idx) => {
                let svc_name = ctx.config.tokengenerator.services
                    .get(service_idx)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let env_name = ctx.config.tokengenerator.services
                    .get(service_idx)
                    .and_then(|s| s.credentials.get(env_idx))
                    .map(|c| c.env.to_string().to_lowercase())
                    .unwrap_or_default();

                ctx.sender.send_app_event(AppLog(LogEntry::new(
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
                let svc_name = ctx.config.tokengenerator.services
                    .get(service_idx)
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let env_name = ctx.config.tokengenerator.services
                    .get(service_idx)
                    .and_then(|s| s.credentials.get(env_idx))
                    .map(|c| c.env.to_string().to_lowercase())
                    .unwrap_or_default();

                self.state.set_token_error(service_idx, env_idx);

                ctx.sender.send_app_event(AppLog(
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

    fn handle_config_event(&mut self, event: TokenGeneratorConfigEvent, ctx: &mut PluginContext) {
        match event {
            ConfigListMove(direction) => {
                use crate::state::token_generator_config::ConfigFocus;
                let len = ctx.config.tokengenerator.services.len();
                let editor = &mut self.config_editor;
                match direction {
                    crate::event::events::Direction::Up => {
                        if editor.config_focus == ConfigFocus::Services {
                            match editor.table_state.selected() {
                                None | Some(0) => {
                                    editor.config_focus = ConfigFocus::Auth0;
                                    editor.table_state.select(None);
                                }
                                _ => editor.table_state.select_previous(),
                            }
                        }
                    }
                    crate::event::events::Direction::Down => {
                        if editor.config_focus == ConfigFocus::Auth0 {
                            if len > 0 {
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
                            ctx.config.tokengenerator.auth0.local = p.local.value().trim().to_string();
                            ctx.config.tokengenerator.auth0.staging = p.staging.value().trim().to_string();
                            ctx.config.tokengenerator.auth0.preproduction = p.preprod.value().trim().to_string();
                            ctx.config.tokengenerator.auth0.production = p.prod.value().trim().to_string();
                            let _ = ctx.config_loader.write_config(ctx.config);
                        }
                        ActiveEdit::Service(p) if p.is_valid() => {
                            let svc = crate::config::model::ServiceConfig {
                                name: p.name.value().trim().to_string(),
                                audience: p.audience.value().trim().to_string(),
                                credentials: p.to_credentials(),
                            };
                            if let Some(idx) = p.edit_index {
                                if let Some(existing) = ctx.config.tokengenerator.services.get_mut(idx) {
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
                if let Some(idx) = self.config_editor.table_state.selected()
                    && idx < ctx.config.tokengenerator.services.len()
                {
                    ctx.config.tokengenerator.services.remove(idx);
                    self.state = TokenGenerator::new(&ctx.config.tokengenerator.services);
                    let new_len = ctx.config.tokengenerator.services.len();
                    if new_len == 0 {
                        self.config_editor.table_state.select(None);
                        ctx.config.enforce_feature_invariants();
                        ctx.sender.send_app_event(RebuildToolList);
                    } else {
                        self.config_editor.table_state.select(Some(idx.min(new_len - 1)));
                    }
                    let _ = ctx.config_loader.write_config(ctx.config);
                }
            }
            ConfigEdit => {
                use crate::state::token_generator_config::ConfigFocus;
                match self.config_editor.config_focus {
                    ConfigFocus::Auth0 => {
                        let auth0 = ctx.config.tokengenerator.auth0.clone();
                        self.config_editor.open_auth0_form(&auth0);
                    }
                    ConfigFocus::Services => {
                        if let Some(idx) = self.config_editor.table_state.selected()
                            && let Some(svc) = ctx.config.tokengenerator.services.get(idx)
                        {
                            let svc = svc.clone();
                            self.config_editor.open_edit_service_form(idx, &svc);
                        }
                    }
                }
            }
            SwitchFocus => {
                use crate::state::token_generator_config::ConfigFocus;
                let editor = &mut self.config_editor;
                editor.config_focus = match editor.config_focus {
                    ConfigFocus::Auth0 => ConfigFocus::Services,
                    ConfigFocus::Services => ConfigFocus::Auth0,
                };
                if editor.config_focus == ConfigFocus::Auth0 {
                    editor.table_state.select(None);
                } else if !ctx.config.tokengenerator.services.is_empty() {
                    editor.table_state.select(Some(0));
                }
            }
        }
    }
}

impl Plugin for TokenGeneratorPlugin {
    fn id(&self)           -> Tool        { Tool::TokenGenerator }
    fn title(&self)        -> &'static str { "M2M Auth0 Token Generator" }
    fn menu_entry(&self)   -> &'static str { "Token Generator" }
    fn config_title(&self) -> &'static str { " Token Generator — Config " }

    fn has_min_config(&self, config: &Config) -> bool {
        !config.tokengenerator.services.is_empty()
    }
    fn is_enabled(&self, features: &Features) -> bool { features.token_generator }
    fn apply_feature_flag(&self, features: &mut Features, enabled: bool) {
        features.token_generator = enabled;
    }

    fn register_bindings(&self, map: &mut KeyEventMap) {
        use crate::state::app::AppFocus;
        use crate::event::events::GenericEvent;

        map.add_static(TokenGenCtx(Focus::Service), KeyCode::Down, KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::ServiceListMove(crate::event::events::Direction::Down)));
        map.add_static(TokenGenCtx(Focus::Service), KeyCode::Up, KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::ServiceListMove(crate::event::events::Direction::Up)));
        map.add_static(TokenGenCtx(Focus::Env), KeyCode::Down, KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::EnvListMove(crate::event::events::Direction::Down)));
        map.add_static(TokenGenCtx(Focus::Env), KeyCode::Up, KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::EnvListMove(crate::event::events::Direction::Up)));
        map.add_static(ToolCtx(Tool::TokenGenerator), KeyCode::Right, KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::SetFocus(Focus::Env)));
        map.add_static(TokenGenCtx(Focus::Service), KeyCode::Left, KeyModifiers::NONE,
            Event::Generic(GenericEvent::SetFocus(AppFocus::List)));
        map.add_static(TokenGenCtx(Focus::Env), KeyCode::Left, KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::SetFocus(Focus::Service)));
        map.add_static(ToolCtx(Tool::TokenGenerator), KeyCode::Enter, KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::GenerateToken));
        map.add_static(ToolCtx(Tool::TokenGenerator), KeyCode::Char('c'), KeyModifiers::NONE,
            Event::Generic(GenericEvent::CopyToClipboard));

        // Config
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Down, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::ConfigListMove(crate::event::events::Direction::Down)));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Up, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::ConfigListMove(crate::event::events::Direction::Up)));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Char('a'), KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::OpenAddService));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Char('e'), KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::ConfigEdit));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Char('x'), KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::RemoveService));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Tab, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::SwitchFocus));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::BackTab, KeyModifiers::SHIFT,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::SwitchFocus));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Left, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));
        map.add_static(ToolConfig(Tool::TokenGenerator), KeyCode::Esc, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));

        // Config editing
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Esc, KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Enter, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::SubmitConfig));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Backspace, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormBackspace));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Left, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormLeft));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Right, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormRight));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Home, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormHome));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::End, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormEnd));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Delete, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormDelete));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Down, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormNextField));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Up, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormPrevField));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::Tab, KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormNextField));
        map.add_static(Editing(Tool::TokenGenerator), KeyCode::BackTab, KeyModifiers::SHIFT,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormPrevField));
        map.add_dynamic(Editing(Tool::TokenGenerator), token_gen_config_form_char);
    }

    fn key_contexts(&self) -> Vec<KeyContext> {
        vec![ToolCtx(Tool::TokenGenerator), TokenGenCtx(self.state.focus)]
    }

    fn config_key_contexts(&self) -> Vec<KeyContext> {
        if self.config_editor.has_open_form() {
            vec![Editing(Tool::TokenGenerator)]
        } else {
            vec![ToolConfig(Tool::TokenGenerator)]
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        widget::render(frame, area, &mut self.state, &config.tokengenerator.services);
    }

    fn render_config(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        config_widget::render(
            frame, area, &mut self.config_editor,
            &config.tokengenerator.auth0, &config.tokengenerator.services,
        );
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut PluginContext) -> bool {
        match event {
            Event::TokenGenerator(e) => { self.handle_tool_event(e.clone(), ctx); true }
            Event::TokenGeneratorConfig(e) => { self.handle_config_event(e.clone(), ctx); true }
            _ => false,
        }
    }

    fn handle_generic_event(&mut self, event: &GenericEvent, ctx: &mut PluginContext) -> bool {
        if *event == CopyToClipboard {
            let token = self.state.token_for_selected_service_env();
            if matches!(token, Token::Ready(_))
                && let Some(value) = token.value()
                && let Err(e) = copy_to_clipboard(value)
            {
                ctx.sender.send_app_event(AppLog(LogEntry::new(
                    LogLevel::Warning,
                    LOG_SOURCE,
                    format!("Copy to clipboard failed: {e}"),
                )));
            }
            true
        } else {
            false
        }
    }

    fn has_open_form(&self) -> bool { self.config_editor.has_open_form() }
    fn close_form(&mut self)        { self.config_editor.close_form(); }

    fn tool_hints(&self) -> (ratatui::text::Line<'static>, ratatui::text::Line<'static>) {
        use crate::ui::styles::{key_desc_style, key_style};
        use ratatui::text::{Line, Span};
        let k = key_style();
        let d = key_desc_style();
        let line2 = match self.state.token_for_selected_service_env() {
            Token::Idle => Line::from(""),
            Token::Requesting => Line::from(vec![Span::styled("Generating token…", key_desc_style())]),
            Token::Ready(_) => Line::from(vec![Span::styled("[c]", k.clone()), Span::styled(" Copy token  ", d.clone())]),
            Token::Error => Line::from(vec![Span::styled("[return]", k.clone()), Span::styled(" Retry  ", d.clone())]),
        };
        (Line::from(vec![
            Span::styled("[↑↓←→]", k.clone()), Span::styled(" Navigate  ", d.clone()),
            Span::styled("[return]", k.clone()), Span::styled(" Generate  ", d.clone()),
            Span::styled("[q]", k.clone()), Span::styled(" Quit", d.clone()),
        ]), line2)
    }

    fn config_hints(&self) -> (ratatui::text::Line<'static>, ratatui::text::Line<'static>) {
        use crate::state::token_generator_config::ConfigFocus;
        use crate::ui::styles::{key_desc_style, key_style};
        use ratatui::text::{Line, Span};
        if self.config_editor.has_open_form() {
            let k = key_style(); let d = key_desc_style();
            return (
                Line::from(vec![
                    Span::styled("[return]", k.clone()), Span::styled(" Save  ", d.clone()),
                    Span::styled("[tab]", k.clone()), Span::styled(" Next field  ", d.clone()),
                    Span::styled("[↑↓]", k.clone()), Span::styled(" Navigate fields  ", d.clone()),
                ]),
                Line::from(vec![Span::styled("[esc]", key_style()), Span::styled(" Cancel  ", key_desc_style())]),
            );
        }
        let k = key_style(); let d = key_desc_style();
        match self.config_editor.config_focus {
            ConfigFocus::Auth0 => (
                Line::from(vec![
                    Span::styled("[a]", k.clone()), Span::styled(" Add  ", d.clone()),
                    Span::styled("[e]", k.clone()), Span::styled(" Edit  ", d.clone()),
                    Span::styled("[q]", k.clone()), Span::styled(" Quit", d.clone()),
                ]),
                Line::from(""),
            ),
            ConfigFocus::Services => {
                let line2 = if self.config_editor.table_state.selected().is_some() {
                    Line::from(vec![
                        Span::styled("[e]", k.clone()), Span::styled(" Edit  ", d.clone()),
                        Span::styled("[x]", k.clone()), Span::styled(" Remove  ", d.clone()),
                    ])
                } else { Line::from("") };
                (Line::from(vec![
                    Span::styled("[↑↓←→]", k.clone()), Span::styled(" Navigate  ", d.clone()),
                    Span::styled("[a]", k.clone()), Span::styled(" Add  ", d.clone()),
                    Span::styled("[q]", k.clone()), Span::styled(" Quit", d.clone()),
                ]), line2)
            }
        }
    }
}

fn token_gen_config_form_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormChar(c)))
}