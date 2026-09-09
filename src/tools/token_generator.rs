pub mod config_editor;
pub(super) mod config_widget;
mod handlers;
pub mod state;
pub(super) mod widget;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::prelude::Rect;
use std::sync::Arc;

use self::config_editor::TokenGeneratorConfigEditor;
use self::state::{Focus, Token, TokenGenerator};
use crate::client::auth_zero::api::AuthZeroApi;
use crate::config::model::{Config, Features};
use crate::event::events::AppEvent::AppLog;
use crate::event::events::GenericEvent::CopyToClipboard;
use crate::event::events::{Event, GenericEvent, TokenGeneratorConfigEvent, TokenGeneratorEvent};
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{
    Editing, TokenGen as TokenGenCtx, Tool as ToolCtx, ToolConfig,
};
use crate::input::key_event_map::KeyEventMap;
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;
use crate::tools::plugin::Plugin;
use crate::utils::string_copy::copy_to_clipboard;

const LOG_SOURCE: LogSource = LogSource::Tool(Tool::TokenGenerator);

pub struct TokenGeneratorPlugin {
    pub(super) state: TokenGenerator,
    pub(super) config_editor: TokenGeneratorConfigEditor,
    pub(super) auth_zero_api: Arc<dyn AuthZeroApi>,
}

impl TokenGeneratorPlugin {
    pub fn new(config: &Config, auth_zero_api: Arc<dyn AuthZeroApi>) -> Self {
        Self {
            state: TokenGenerator::new(&config.tokengenerator.services),
            config_editor: TokenGeneratorConfigEditor::new(),
            auth_zero_api,
        }
    }
}
impl Plugin for TokenGeneratorPlugin {
    fn id(&self) -> Tool {
        Tool::TokenGenerator
    }
    fn title(&self) -> &'static str {
        "M2M Auth0 Token Generator"
    }
    fn menu_entry(&self) -> &'static str {
        "Token Generator"
    }
    fn config_title(&self) -> &'static str {
        " Token Generator — Config "
    }

    fn has_min_config(&self, config: &Config) -> bool {
        !config.tokengenerator.services.is_empty()
    }
    fn is_enabled(&self, features: &Features) -> bool {
        features.token_generator
    }
    fn apply_feature_flag(&self, features: &mut Features, enabled: bool) {
        features.token_generator = enabled;
    }

    fn register_bindings(&self, map: &mut KeyEventMap) {
        use crate::event::events::GenericEvent;
        use crate::state::app::AppFocus;

        map.add_static(
            TokenGenCtx(Focus::Service),
            KeyCode::Down,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::ServiceListMove(
                crate::event::events::Direction::Down,
            )),
        );
        map.add_static(
            TokenGenCtx(Focus::Service),
            KeyCode::Up,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::ServiceListMove(
                crate::event::events::Direction::Up,
            )),
        );
        map.add_static(
            TokenGenCtx(Focus::Audience),
            KeyCode::Down,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::AudienceListMove(
                crate::event::events::Direction::Down,
            )),
        );
        map.add_static(
            TokenGenCtx(Focus::Audience),
            KeyCode::Up,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::AudienceListMove(
                crate::event::events::Direction::Up,
            )),
        );
        map.add_static(
            TokenGenCtx(Focus::Env),
            KeyCode::Down,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::EnvListMove(
                crate::event::events::Direction::Down,
            )),
        );
        map.add_static(
            TokenGenCtx(Focus::Env),
            KeyCode::Up,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::EnvListMove(
                crate::event::events::Direction::Up,
            )),
        );
        // Right/Left are resolved dynamically (see handle_tool_event) so the
        // Audience column can be skipped when the selected service has only one
        // audience — the target focus depends on config state, not just the key.
        map.add_static(
            TokenGenCtx(Focus::Service),
            KeyCode::Right,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::FocusRight),
        );
        map.add_static(
            TokenGenCtx(Focus::Audience),
            KeyCode::Right,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::FocusRight),
        );
        map.add_static(
            TokenGenCtx(Focus::Service),
            KeyCode::Left,
            KeyModifiers::NONE,
            Event::Generic(GenericEvent::SetFocus(AppFocus::List)),
        );
        map.add_static(
            TokenGenCtx(Focus::Audience),
            KeyCode::Left,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::SetFocus(Focus::Service)),
        );
        map.add_static(
            TokenGenCtx(Focus::Env),
            KeyCode::Left,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::FocusLeft),
        );
        map.add_static(
            ToolCtx(Tool::TokenGenerator),
            KeyCode::Enter,
            KeyModifiers::NONE,
            Event::TokenGenerator(TokenGeneratorEvent::GenerateToken),
        );
        map.add_static(
            ToolCtx(Tool::TokenGenerator),
            KeyCode::Char('c'),
            KeyModifiers::NONE,
            Event::Generic(GenericEvent::CopyToClipboard),
        );

        // Config
        map.add_static(
            ToolConfig(Tool::TokenGenerator),
            KeyCode::Down,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::ConfigListMove(
                crate::event::events::Direction::Down,
            )),
        );
        map.add_static(
            ToolConfig(Tool::TokenGenerator),
            KeyCode::Up,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::ConfigListMove(
                crate::event::events::Direction::Up,
            )),
        );
        map.add_static(
            ToolConfig(Tool::TokenGenerator),
            KeyCode::Char('a'),
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::OpenAddService),
        );
        map.add_static(
            ToolConfig(Tool::TokenGenerator),
            KeyCode::Char('e'),
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::ConfigEdit),
        );
        map.add_static(
            ToolConfig(Tool::TokenGenerator),
            KeyCode::Char('x'),
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::RemoveService),
        );
        map.add_static(
            ToolConfig(Tool::TokenGenerator),
            KeyCode::Left,
            KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig),
        );
        map.add_static(
            ToolConfig(Tool::TokenGenerator),
            KeyCode::Esc,
            KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig),
        );

        // Config editing
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Esc,
            KeyModifiers::NONE,
            Event::App(crate::event::events::AppEvent::CloseToolConfig),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Enter,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::SubmitConfig),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Backspace,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormBackspace),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Left,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormLeft),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Right,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormRight),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Home,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormHome),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::End,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormEnd),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Delete,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormDelete),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Down,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormNextField),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Up,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormPrevField),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::Tab,
            KeyModifiers::NONE,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormNextField),
        );
        map.add_static(
            Editing(Tool::TokenGenerator),
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
            Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormPrevField),
        );
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
        widget::render(
            frame,
            area,
            &mut self.state,
            &config.tokengenerator.services,
        );
    }

    fn render_config(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        config_widget::render(
            frame,
            area,
            &mut self.config_editor,
            &config.tokengenerator.auth0,
            &config.tokengenerator.services,
        );
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut PluginContext) -> bool {
        match event {
            Event::TokenGenerator(e) => {
                self.handle_tool_event(e.clone(), ctx);
                true
            }
            Event::TokenGeneratorConfig(e) => {
                self.handle_config_event(e.clone(), ctx);
                true
            }
            _ => false,
        }
    }

    fn handle_generic_event(&mut self, event: &GenericEvent, ctx: &mut PluginContext) -> bool {
        if *event == CopyToClipboard {
            let token = self.state.token_for_selected_service_audience_env();
            if matches!(token, Token::Ready(_))
                && let Some(value) = token.value()
                && let Err(e) = copy_to_clipboard(value)
            {
                ctx.sender.send(AppLog(LogEntry::new(
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

    fn has_open_form(&self) -> bool {
        self.config_editor.has_open_form()
    }
    fn close_form(&mut self) {
        self.config_editor.close_form();
    }

    fn normalize_config(&self, config: &mut Config) {
        let a = &mut config.tokengenerator.auth0;
        a.local = a.local.trim_end_matches('/').to_string();
        a.staging = a.staging.trim_end_matches('/').to_string();
        a.preproduction = a.preproduction.trim_end_matches('/').to_string();
        a.production = a.production.trim_end_matches('/').to_string();
    }

    fn tool_hints(&self) -> (ratatui::text::Line<'static>, ratatui::text::Line<'static>) {
        use crate::ui::styles::{key_desc_style, key_style};
        use ratatui::text::{Line, Span};
        let k = key_style();
        let d = key_desc_style();
        let line2 = match self.state.token_for_selected_service_audience_env() {
            Token::Idle => Line::from(""),
            Token::Requesting => {
                Line::from(vec![Span::styled("Generating token…", key_desc_style())])
            }
            Token::Ready(_) => Line::from(vec![
                Span::styled("[c]", k),
                Span::styled(" Copy token  ", d),
            ]),
            Token::Error => Line::from(vec![
                Span::styled("[return]", k),
                Span::styled(" Retry  ", d),
            ]),
        };
        (
            Line::from(vec![
                Span::styled("[↑↓←→]", k),
                Span::styled(" Navigate  ", d),
                Span::styled("[return]", k),
                Span::styled(" Generate  ", d),
                Span::styled("[q]", k),
                Span::styled(" Quit", d),
            ]),
            line2,
        )
    }

    fn config_hints(&self) -> (ratatui::text::Line<'static>, ratatui::text::Line<'static>) {
        use crate::ui::styles::{key_desc_style, key_style};
        use ratatui::text::{Line, Span};
        if self.config_editor.has_open_form() {
            let k = key_style();
            let d = key_desc_style();
            return (
                Line::from(vec![
                    Span::styled("[return]", k),
                    Span::styled(" Save  ", d),
                    Span::styled("[tab]", k),
                    Span::styled(" Next field  ", d),
                    Span::styled("[↑↓]", k),
                    Span::styled(" Navigate fields  ", d),
                ]),
                Line::from(vec![
                    Span::styled("[esc]", key_style()),
                    Span::styled(" Cancel  ", key_desc_style()),
                ]),
            );
        }
        let k = key_style();
        let d = key_desc_style();
        if self.config_editor.is_auth0_selected() {
            (
                Line::from(vec![
                    Span::styled("[↑↓]", k),
                    Span::styled(" Navigate  ", d),
                    Span::styled("[e]", k),
                    Span::styled(" Edit auth0 endpoints  ", d),
                    Span::styled("[a]", k),
                    Span::styled(" Add service  ", d),
                    Span::styled("[q]", k),
                    Span::styled(" Quit", d),
                ]),
                Line::from(""),
            )
        } else {
            (
                Line::from(vec![
                    Span::styled("[↑↓]", k),
                    Span::styled(" Navigate  ", d),
                    Span::styled("[a]", k),
                    Span::styled(" Add service  ", d),
                    Span::styled("[q]", k),
                    Span::styled(" Quit", d),
                ]),
                Line::from(vec![
                    Span::styled("[e]", k),
                    Span::styled(" Edit service  ", d),
                    Span::styled("[x]", k),
                    Span::styled(" Remove  ", d),
                ]),
            )
        }
    }
}

fn token_gen_config_form_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| Event::TokenGeneratorConfig(TokenGeneratorConfigEvent::FormChar(c)))
}
