use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
use crate::client::auth_zero::api::{AuthZeroApi, ImmediateAuthZeroApi};
use crate::client::healthcheck::api::{HealthcheckApi, ImmediateHealthcheckApi};
use crate::client::jira::api::{ImmediateJiraApi, JiraApi};
use crate::config::loader::ConfigLoader;
use crate::config::model::Config;
use crate::events::event::AppEvent::*;
use crate::events::event::{AppEvent, Event};
use crate::events::handler::EventHandler;
use crate::events::sender::EventSender;
use crate::events::tools::{jira, service_status, token_generator};
use crate::input::key_bindings::register_bindings;
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{
    Editing, Error, Global, List, TokenGen, ToolConfigEditing, ToolIgnore,
};
use crate::input::key_event_map::KeyEventMap;
pub(crate) use crate::state::app::{AppFocus, Tool};
use crate::utils::overlay::overlay_area;
use crate::utils::update_list_state;
use crate::{state::app::AppState, ui::layout, ui::widgets::*};
use crossterm::event::{self, KeyEvent, KeyEventKind};
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::sync::Arc;
use std::time::Duration;

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    pub(crate) state: AppState,
    event_handler: EventHandler,
    pub(crate) event_sender: EventSender,
    pub(crate) config: Config,
    config_loader: ConfigLoader,
    key_event_map: KeyEventMap,

    // External Services
    pub(crate) jira_api: Arc<dyn JiraApi>,
    pub(crate) auth_zero_api: Arc<dyn AuthZeroApi>,
    pub(crate) healthcheck_api: Arc<dyn HealthcheckApi>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config, config_loader: ConfigLoader) -> Self {
        let event_handler = EventHandler::new();
        let event_sender = event_handler.sender();
        Self {
            running: true,
            state: AppState::new(&config),
            event_handler,
            event_sender,
            config,
            config_loader,
            key_event_map: KeyEventMap::default(),

            // wire real infra
            jira_api: Arc::new(ImmediateJiraApi::new()),
            auth_zero_api: Arc::new(ImmediateAuthZeroApi::new()),
            healthcheck_api: Arc::new(ImmediateHealthcheckApi::new()),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let async_sender = self.event_sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_mins(15));
            loop {
                interval.tick().await; // This should go first.
                async_sender.send(ScanServices);
                async_sender.send(ScanTickets);
            }
        });

        // Register bindings
        register_bindings(&mut self.key_event_map);

        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            match self.event_handler.next().await? {
                Event::Tick => {}
                Event::Crossterm(event) => match event {
                    event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => self.handle_app_event(app_event),
            }
        }
        Ok(())
    }

    fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            // global
            Quit => self.running = false,
            SetFocus(focus) => self.state.focus = focus,
            ListSelect(tool_state) => self.state.current_tool = tool_state,
            ListMove(direction) => {
                let tool_list = &mut self.state.tool_list;
                update_list_state::update_list(
                    &mut tool_list.list_state,
                    direction,
                    tool_list.items.len(),
                );
                if let Some(index) = tool_list.list_state.selected()
                    && let Some(tool) = tool_list.items.get(index).cloned()
                {
                    self.event_sender.send(ListSelect(tool))
                }
            }
            SystemError(error) => self.state.error = Some(error),
            DismissError => self.state.error = None,

            CopyToClipboard => match self.state.current_tool {
                ServiceStatus => service_status::handle_event(self, CopyToClipboard),
                TokenGenerator => token_generator::handle_event(self, CopyToClipboard),
                _ => {}
            },
            OpenInBrowser => match self.state.current_tool {
                ServiceStatus => service_status::handle_event(self, OpenInBrowser),
                Jira => jira::handle_event(self, OpenInBrowser),
                _ => {}
            },

            // service status events
            e @ ServiceStatusListMove(..)
            | e @ ScanServices
            | e @ ScanServiceEnv(..)
            | e @ GetCommitRefOk(..)
            | e @ GetCommitRefErrored(..) => service_status::handle_event(self, e),
            // token generator events
            e @ TokenGenEnvListMove(..)
            | e @ TokenGenServiceListMove(..)
            | e @ SetTokenGenFocus(..)
            | e @ GenerateToken
            | e @ TokenGenerated(..)
            | e @ TokenFailed(..) => token_generator::handle_event(self, e),
            // jira ticket events
            e @ JiraTicketListMove(..)
            | e @ NewJiraTicket
            | e @ AddTicketIdChar(..)
            | e @ RemoveTicketIdChar
            | e @ SubmitTicketId
            | e @ TicketRetrieved(..)
            | e @ RemoveTicket
            | e @ JiraTicketMove(..)
            | e @ JiraTicketListUpdate
            | e @ ScanTickets => jira::handle_event(self, e),

            // config panel events
            ConfigListMove(direction) => {
                let editor = &mut self.state.config_editor;
                update_list_state::update_list(
                    &mut editor.list_state,
                    direction,
                    editor.items.len(),
                );
            }
            ToggleFeature => {
                let has_jira_config = self.config.jira.is_some();
                if self.state.config_editor.toggle_selected().is_some() {
                    self.config.features = self.state.config_editor.to_features();
                    // Best-effort write — silently ignore errors so the app stays usable.
                    let _ = self.config_loader.write_config(&self.config);
                    self.state.rebuild_tool_list(has_jira_config);
                }
            }
            OpenToolConfig(_) => {
                // Use the config editor's currently selected tool rather than the event payload
                if let Some(idx) = self.state.config_editor.list_state.selected()
                    && let Some(item) = self.state.config_editor.items.get(idx)
                {
                    self.state.focus = AppFocus::ToolConfig(item.tool);
                }
            }
            CloseToolConfig => {
                // Close inline edit form if open, otherwise exit tool config.
                let ss_form_open = self.state.service_status_config_editor.form.is_some();
                let tg_form_open = self.state.token_generator_config_editor.form.is_some();
                let jira_form_open = self.state.jira_config_editor.form.is_some();
                if ss_form_open {
                    self.state.service_status_config_editor.form = None;
                } else if tg_form_open {
                    self.state.token_generator_config_editor.form = None;
                } else if jira_form_open {
                    self.state.jira_config_editor.form = None;
                } else {
                    self.state.focus = AppFocus::Config;
                }
            }
            // Service Status config events
            e @ ServiceStatusConfigListMove(..)
            | e @ OpenAddService
            | e @ OpenEditService
            | e @ ServiceStatusFormNextField
            | e @ ServiceStatusFormPrevField
            | e @ ServiceStatusFormChar(..)
            | e @ ServiceStatusFormBackspace
            | e @ SubmitServiceConfig
            | e @ RemoveService => self.handle_service_status_config_event(e),
            // Token Generator config events
            e @ TokenGenConfigListMove(..)
            | e @ OpenAddTokenGenService
            | e @ TokenGenConfigFormNextField
            | e @ TokenGenConfigFormPrevField
            | e @ TokenGenConfigFormChar(..)
            | e @ TokenGenConfigFormBackspace
            | e @ SubmitTokenGenConfig
            | e @ TgConfigSwitchFocus
            | e @ TgConfigEdit
            | e @ RemoveTokenGenService => self.handle_token_gen_config_event(e),
            // Jira config events
            e @ OpenJiraConfigEdit
            | e @ JiraConfigFormNextField
            | e @ JiraConfigFormPrevField
            | e @ JiraConfigFormChar(..)
            | e @ JiraConfigFormBackspace
            | e @ SubmitJiraConfig => self.handle_jira_config_event(e),
        }
    }

    fn handle_service_status_config_event(&mut self, event: AppEvent) {
        let editor = &mut self.state.service_status_config_editor;
        match event {
            ServiceStatusConfigListMove(direction) => {
                let len = self.config.servicestatus.len();
                let state = &mut self.state.service_status_config_editor.table_state;
                if len == 0 {
                    state.select(None);
                } else {
                    match direction {
                        crate::events::event::Direction::Up => match state.selected() {
                            None | Some(0) => state.select(None),
                            _ => state.select_previous(),
                        },
                        crate::events::event::Direction::Down => {
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
                if let Some(idx) = self
                    .state
                    .service_status_config_editor
                    .table_state
                    .selected()
                    && let Some(svc) = self.config.servicestatus.get(idx)
                {
                    self.state
                        .service_status_config_editor
                        .open_edit_form(idx, svc);
                }
            }
            ServiceStatusFormNextField => {
                if let Some(form) = &mut editor.form {
                    form.active_field = form.active_field.next();
                }
            }
            ServiceStatusFormPrevField => {
                if let Some(form) = &mut editor.form {
                    form.active_field = form.active_field.prev();
                }
            }
            ServiceStatusFormChar(c) => {
                if let Some(form) = &mut editor.form {
                    form.active_field_value_mut().push(c);
                }
            }
            ServiceStatusFormBackspace => {
                if let Some(form) = &mut editor.form {
                    form.active_field_value_mut().pop();
                }
            }
            SubmitServiceConfig => {
                if let Some(form) = self.state.service_status_config_editor.form.take()
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
                        if let Some(existing) = self.config.servicestatus.get_mut(idx) {
                            *existing = service;
                        }
                    } else {
                        // Add new
                        self.config.servicestatus.push(service);
                    }
                    self.state.service_status = crate::state::service_status::ServiceStatus::new(
                        self.config.servicestatus.len(),
                    );
                    let _ = self.config_loader.write_config(&self.config);
                }
                // If invalid, just close the form without saving
            }
            RemoveService => {
                if let Some(idx) = self
                    .state
                    .service_status_config_editor
                    .table_state
                    .selected()
                    && idx < self.config.servicestatus.len()
                {
                    self.config.servicestatus.remove(idx);
                    self.state.service_status = crate::state::service_status::ServiceStatus::new(
                        self.config.servicestatus.len(),
                    );
                    // Clamp selection
                    let new_len = self.config.servicestatus.len();
                    if new_len == 0 {
                        self.state
                            .service_status_config_editor
                            .table_state
                            .select(None);
                    } else {
                        let clamped = idx.min(new_len - 1);
                        self.state
                            .service_status_config_editor
                            .table_state
                            .select(Some(clamped));
                    }
                    let _ = self.config_loader.write_config(&self.config);
                }
            }
            _ => {}
        }
    }

    fn handle_token_gen_config_event(&mut self, event: AppEvent) {
        use crate::state::token_generator_config::ActiveEdit;
        match event {
            TokenGenConfigListMove(direction) => {
                use crate::state::token_generator_config::ConfigFocus;
                let len = self.config.tokengenerator.services.len();
                let editor = &mut self.state.token_generator_config_editor;
                match direction {
                    crate::events::event::Direction::Up => {
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
                    crate::events::event::Direction::Down => {
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
            OpenAddTokenGenService => {
                self.state
                    .token_generator_config_editor
                    .open_add_service_form();
            }
            TokenGenConfigFormNextField => {
                match &mut self.state.token_generator_config_editor.form {
                    Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.next(),
                    Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.next(),
                    None => {}
                }
            }
            TokenGenConfigFormPrevField => {
                match &mut self.state.token_generator_config_editor.form {
                    Some(ActiveEdit::Auth0(p)) => p.active_field = p.active_field.prev(),
                    Some(ActiveEdit::Service(p)) => p.active_field = p.active_field.prev(),
                    None => {}
                }
            }
            TokenGenConfigFormChar(c) => match &mut self.state.token_generator_config_editor.form {
                Some(ActiveEdit::Auth0(p)) => p.active_field_value_mut().push(c),
                Some(ActiveEdit::Service(p)) => p.active_field_value_mut().push(c),
                None => {}
            },
            TokenGenConfigFormBackspace => {
                match &mut self.state.token_generator_config_editor.form {
                    Some(ActiveEdit::Auth0(p)) => {
                        p.active_field_value_mut().pop();
                    }
                    Some(ActiveEdit::Service(p)) => {
                        p.active_field_value_mut().pop();
                    }
                    None => {}
                }
            }
            SubmitTokenGenConfig => {
                if let Some(form) = self.state.token_generator_config_editor.form.take() {
                    match form {
                        ActiveEdit::Auth0(p) => {
                            self.config.tokengenerator.auth0.local = p.local.trim().to_string();
                            self.config.tokengenerator.auth0.staging = p.staging.trim().to_string();
                            self.config.tokengenerator.auth0.preproduction =
                                p.preprod.trim().to_string();
                            self.config.tokengenerator.auth0.production = p.prod.trim().to_string();
                            let _ = self.config_loader.write_config(&self.config);
                        }
                        ActiveEdit::Service(p) if p.is_valid() => {
                            let svc = crate::config::model::ServiceConfig {
                                name: p.name.trim().to_string(),
                                audience: p.audience.trim().to_string(),
                                credentials: p.to_credentials(),
                            };
                            if let Some(idx) = p.edit_index {
                                if let Some(existing) =
                                    self.config.tokengenerator.services.get_mut(idx)
                                {
                                    *existing = svc;
                                }
                            } else {
                                self.config.tokengenerator.services.push(svc);
                            }
                            self.state.token_generator =
                                crate::state::token_generator::TokenGenerator::new(
                                    &self.config.tokengenerator.services,
                                );
                            let _ = self.config_loader.write_config(&self.config);
                        }
                        _ => {} // invalid service form — close without saving
                    }
                }
            }
            RemoveTokenGenService => {
                if let Some(idx) = self
                    .state
                    .token_generator_config_editor
                    .table_state
                    .selected()
                    && idx < self.config.tokengenerator.services.len()
                {
                    self.config.tokengenerator.services.remove(idx);
                    self.state.token_generator = crate::state::token_generator::TokenGenerator::new(
                        &self.config.tokengenerator.services,
                    );
                    let new_len = self.config.tokengenerator.services.len();
                    if new_len == 0 {
                        self.state
                            .token_generator_config_editor
                            .table_state
                            .select(None);
                    } else {
                        self.state
                            .token_generator_config_editor
                            .table_state
                            .select(Some(idx.min(new_len - 1)));
                    }
                    let _ = self.config_loader.write_config(&self.config);
                }
            }
            TgConfigEdit => {
                use crate::state::token_generator_config::ConfigFocus;
                let editor = &self.state.token_generator_config_editor;
                match editor.config_focus {
                    ConfigFocus::Auth0 => {
                        let auth0 = self.config.tokengenerator.auth0.clone();
                        self.state
                            .token_generator_config_editor
                            .open_auth0_form(&auth0);
                    }
                    ConfigFocus::Services => {
                        if let Some(idx) = self
                            .state
                            .token_generator_config_editor
                            .table_state
                            .selected()
                            && let Some(svc) = self.config.tokengenerator.services.get(idx)
                        {
                            let svc = svc.clone();
                            self.state
                                .token_generator_config_editor
                                .open_edit_service_form(idx, &svc);
                        }
                    }
                }
            }
            TgConfigSwitchFocus => {
                use crate::state::token_generator_config::ConfigFocus;
                let editor = &mut self.state.token_generator_config_editor;
                editor.config_focus = match editor.config_focus {
                    ConfigFocus::Auth0 => ConfigFocus::Services,
                    ConfigFocus::Services => ConfigFocus::Auth0,
                };
                if editor.config_focus == ConfigFocus::Auth0 {
                    editor.table_state.select(None);
                } else if !self.config.tokengenerator.services.is_empty() {
                    editor.table_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn handle_jira_config_event(&mut self, event: AppEvent) {
        match event {
            OpenJiraConfigEdit => {
                self.state
                    .jira_config_editor
                    .open_form(self.config.jira.as_ref());
            }
            JiraConfigFormNextField => {
                if let Some(p) = &mut self.state.jira_config_editor.form {
                    p.active_field = p.active_field.next();
                }
            }
            JiraConfigFormPrevField => {
                if let Some(p) = &mut self.state.jira_config_editor.form {
                    p.active_field = p.active_field.prev();
                }
            }
            JiraConfigFormChar(c) => {
                if let Some(p) = &mut self.state.jira_config_editor.form {
                    p.active_field_value_mut().push(c);
                }
            }
            JiraConfigFormBackspace => {
                if let Some(p) = &mut self.state.jira_config_editor.form {
                    p.active_field_value_mut().pop();
                }
            }
            SubmitJiraConfig => {
                if let Some(form) = self.state.jira_config_editor.form.take() {
                    if form.is_empty() {
                        self.config.jira = None;
                    } else {
                        self.config.jira = Some(crate::config::model::JiraConfig {
                            url: form.url.trim().to_string(),
                            email: form.email.trim().to_string(),
                            token: form.token.trim().to_string(),
                        });
                    }
                    let has_jira_config = self.config.jira.is_some();
                    self.state.rebuild_tool_list(has_jira_config);
                    let _ = self.config_loader.write_config(&self.config);
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area());

        list::render(frame, areas.tools_list, &mut self.state);

        config_list::render(frame, areas.config_list, &mut self.state);

        tool::render(frame, areas.content, &mut self.state, &self.config);

        footer::render(frame, areas.footer, &self.state);

        if let Some(error) = &self.state.error {
            let red = Style::default().fg(Color::Red);
            let block = Block::bordered()
                .title(format!(" {} ", error.title))
                .border_style(red)
                .title_style(red);
            let lines: Vec<Line> = vec![
                Line::from(format!("{}: {}", error.tool, error.originating_event)),
                Line::from(""),
                Line::from(error.description.clone()),
            ];

            let paragraph = Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(block)
                .alignment(Alignment::Left);

            let area = overlay_area(frame.area(), 50, 7);
            frame.render_widget(Clear, area);
            frame.render_widget(paragraph, area);
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        // First-match-wins: the most specific context in the stack takes priority.
        // This prevents lower-priority contexts (e.g. Global Quit on Esc) from
        // also firing when a higher-priority context already handled the key.
        for context in self.get_context_stack() {
            if let Some(event) = self.key_event_map.resolve(context, key) {
                self.event_sender.send(event.clone());
                break;
            }
        }

        Ok(())
    }

    fn get_context_stack(&self) -> Vec<KeyContext> {
        let mut stack = Vec::new();

        // If an Error pop up is displayed, don't allow additional contexts i.e
        // disable all key contexts except global and the error form.
        if self.state.error.is_some() {
            stack.push(Error);
        } else {
            match self.state.focus {
                AppFocus::List => {
                    stack.push(List);
                }
                AppFocus::Tool => {
                    stack.push(KeyContext::Tool(self.state.current_tool));
                    if self.state.current_tool == TokenGenerator {
                        stack.push(TokenGen(self.state.token_generator.focus))
                    } else {
                        stack.push(ToolIgnore(TokenGenerator));
                    }
                }
                AppFocus::ToolConfig(tool) => {
                    // Use editing context when inline edit form is open
                    if tool == ServiceStatus
                        && self.state.service_status_config_editor.form.is_some()
                    {
                        stack.push(Editing(ServiceStatus));
                    } else if tool == TokenGenerator
                        && self.state.token_generator_config_editor.form.is_some()
                    {
                        stack.push(Editing(TokenGenerator));
                    } else if tool == Jira && self.state.jira_config_editor.form.is_some() {
                        stack.push(ToolConfigEditing(Jira));
                    } else {
                        stack.push(KeyContext::ToolConfig(tool));
                    }
                }
                AppFocus::Config => {
                    stack.push(KeyContext::Config);
                }
                AppFocus::JiraInput => {
                    stack.push(Editing(Jira));
                }
            }
        }

        stack.push(Global);
        stack
    }
}
