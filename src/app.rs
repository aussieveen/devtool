use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
use crate::client::auth_zero::api::{AuthZeroApi, ImmediateAuthZeroApi};
use crate::client::healthcheck::api::{HealthcheckApi, ImmediateHealthcheckApi};
use crate::client::jira::api::{ImmediateJiraApi, JiraApi};
use crate::config::loader::ConfigLoader;
use crate::config::model::Config;
use crate::event::event::AppEvent::*;
use crate::event::event::{AppEvent, Event, GenericEvent, ServiceStatusEvent};
use crate::event::handler::EventHandler;
use crate::event::sender::EventSender;
use crate::event::handlers::tools::{jira, service_status, token_generator};
use crate::event::handlers::config::{
    jira as jira_config, service_status as service_status_config,
    token_generator as token_generator_config
};
use crate::input::key_bindings::register_bindings;
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{
    Editing, Error, Global, List, Logs, TokenGen, ToolConfigEditing, ToolIgnore,
};
use crate::input::key_event_map::KeyEventMap;
pub(crate) use crate::state::app::{AppFocus, Tool};
use crate::utils::overlay::overlay_area;
use crate::utils::update_list_state;
use crate::{state::app::AppState, ui::layout, ui::widgets::*};
use crossterm::event::{self, KeyEvent, KeyEventKind};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::sync::Arc;
use std::time::Duration;
use crate::event::event::GenericEvent::{CopyToClipboard, OpenInBrowser};
use crate::event::event::JiraEvent::ScanTickets;
use crate::event::event::ServiceStatusEvent::Scan;

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    pub(crate) state: AppState,
    event_handler: EventHandler,
    pub(crate) event_sender: EventSender,
    pub(crate) config: Config,
    pub(crate) config_loader: ConfigLoader,
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
        // Log app startup
        self.state.log.push_log(
            crate::state::log::LogLevel::Info,
            "app".to_string(),
            "App started — config loaded".to_string(),
        );

        let async_sender = self.event_sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_mins(15));
            loop {
                interval.tick().await; // This should go first.
                async_sender.send_service_status_event(Scan);
                async_sender.send_jira_event(ScanTickets);
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
                Event::App(event) => self.handle_app_event(event),
                Event::ServiceStatus(event ) => service_status::handle_event(&mut self, event),
                Event::ServiceStatusConfig(event) => service_status_config::handle_event(&mut self, event),
                Event::TokenGenerator(event) => {token_generator::handle_event(&mut self, event)}
                Event::TokenGeneratorConfig(event) => {token_generator_config::handle_event(&mut self, event)}
                Event::Jira(event) => {jira::handle_event(&mut self, event)}
                Event::JiraConfig(event) => {jira_config::handle_event(&mut self, event)}
            }
        }
        Ok(())
    }

    fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            // global
            Quit => self.running = false,
            SetFocus(focus) => self.state.focus = focus,
            OpenLogs => {
                self.state.focus = AppFocus::Logs;
                if self.state.log.selected_item == crate::state::log::LogsItem::Activity {
                    self.state.log.mark_activity_seen();
                }
            }
            LogsListMove(direction) => {
                use crate::event::event::Direction;
                match direction {
                    Direction::Down => {
                        self.state.log.select_next();
                        if self.state.log.selected_item == crate::state::log::LogsItem::Activity {
                            self.state.log.mark_activity_seen();
                        }
                    }
                    Direction::Up => {
                        self.state.log.select_prev();
                        if self.state.log.selected_item == crate::state::log::LogsItem::Activity {
                            self.state.log.mark_activity_seen();
                        }
                    }
                }
            }
            ActivityEvent(source, message) => {
                self.state.log.push_activity(source, message);
            }
            AppLog(level, source, message) => {
                self.state.log.push_log(level, source, message);
            }
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
                    self.event_sender.send_app_event(ListSelect(tool))
                }
            }
            SystemError(error) => self.state.error = Some(error),
            DismissError => self.state.error = None,

            // token generator event
            e @ TokenGenEnvListMove(..)
            | e @ TokenGenServiceListMove(..)
            | e @ SetTokenGenFocus(..)
            | e @ GenerateToken
            | e @ TokenGenerated(..)
            | e @ TokenFailed(..) => token_generator::handle_event(self, e),
            // jira ticket event
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

            // config panel event
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
                if let Some((tool, now_enabled)) = self.state.config_editor.toggle_selected() {
                    // If the user is trying to enable a feature, check minimum config is present.
                    // If not, revert the toggle so the feature stays disabled.
                    let has_min_config = match tool {
                        ServiceStatus => !self.config.servicestatus.is_empty(),
                        TokenGenerator => !self.config.tokengenerator.services.is_empty(),
                        Jira => self.config.jira.is_some(),
                    };
                    if now_enabled && !has_min_config {
                        // Revert — toggle back to disabled without persisting.
                        self.state.config_editor.toggle_selected();
                    } else {
                        self.config.features = self.state.config_editor.to_features();
                        // Best-effort write — silently ignore errors so the app stays usable.
                        let _ = self.config_loader.write_config(&self.config);
                        self.state.rebuild_tool_list(has_jira_config);
                    }
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
            // Service Status config event
            e @ ServiceStatusConfigListMove(..)
            | e @ OpenAddService
            | e @ OpenEditService
            | e @ ServiceStatusFormNextField
            | e @ ServiceStatusFormPrevField
            | e @ ServiceStatusFormChar(..)
            | e @ ServiceStatusFormBackspace
            | e @ SubmitServiceConfig
            | e @ RemoveService => self.handle_service_status_config_event(e),
            // Token Generator config event
            e @ TokenGenConfigListMove(..)
            | e @ OpenAddTokenGenService
            | e @ TokenGenConfigFormNextField
            | e @ TokenGenConfigFormPrevField
            | e @ TokenGenConfigFormChar(..)
            | e @ TokenGenConfigFormBackspace
            | e @ SubmitTokenGenConfig
            | e @ TokenGeneratorConfigSwitchFocus
            | e @ TokenGeneratorConfigEdit
            | e @ RemoveTokenGenService => self.handle_token_gen_config_event(e),
            // Jira config event
            e @ OpenJiraConfigEdit
            | e @ JiraConfigFormNextField
            | e @ JiraConfigFormPrevField
            | e @ JiraConfigFormChar(..)
            | e @ JiraConfigFormBackspace
            | e @ SubmitJiraConfig => self.handle_jira_config_event(e),
        }
    }

    fn handle_generic_event(&mut self, event: GenericEvent){
        match event {
            CopyToClipboard => match self.state.current_tool {
                ServiceStatus => service_status::handle_generic_event(self, CopyToClipboard),
                TokenGenerator => token_generator::handle_event(self, CopyToClipboard),
                _ => {}
            },
            OpenInBrowser => match self.state.current_tool {
                ServiceStatus => service_status::handle_generic_event(self, OpenInBrowser),
                Jira => jira::handle_event(self, OpenInBrowser),
                _ => {}
            },
        }
    }

    fn handle_service_status_config_event(&mut self, event: AppEvent) {

    }

    fn handle_token_gen_config_event(&mut self, event: AppEvent) {
        use crate::state::token_generator_config::ActiveEdit;
        match event {
            TokenGenConfigListMove(direction) => {
                use crate::state::token_generator_config::ConfigFocus;
                let len = self.config.tokengenerator.services.len();
                let editor = &mut self.state.token_generator_config_editor;
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
                        // Auto-disable the feature since there's no backing config left.
                        self.config.features.token_generator = false;
                        self.state
                            .config_editor
                            .sync_from_features(&self.config.features);
                        self.state.rebuild_tool_list(self.config.jira.is_some());
                    } else {
                        self.state
                            .token_generator_config_editor
                            .table_state
                            .select(Some(idx.min(new_len - 1)));
                    }
                    let _ = self.config_loader.write_config(&self.config);
                }
            }
            TokenGeneratorConfigEdit => {
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
            TokenGeneratorConfigSwitchFocus => {
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

    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area());

        list::render(frame, areas.tools_list, &mut self.state);

        config_list::render(frame, areas.config_list, &mut self.state);

        logs_list::render(frame, areas.logs_list, &mut self.state);

        if matches!(self.state.focus, AppFocus::Logs) {
            let focused = true;
            tools::logs::render(frame, areas.content, &self.state.log, focused);
        } else {
            tool::render(frame, areas.content, &mut self.state, &self.config);
        }

        footer::render(frame, areas.footer, &self.state);

        if let Some(error) = &self.state.error {
            let red = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
            let dim = Style::default().add_modifier(Modifier::DIM);
            let key = crate::ui::styles::key_style();

            let block = Block::bordered().border_style(red).title_style(red);
            let content = Paragraph::new(vec![
                Line::from(Span::styled(error.title.clone(), red)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("See ", dim),
                    Span::styled("[3]", key),
                    Span::styled(" Logs for details  ", dim),
                    Span::styled("[d]", key),
                    Span::styled(" Dismiss", dim),
                ]),
            ])
            .block(block);

            let area = overlay_area(frame.area(), 40, 5);
            frame.render_widget(Clear, area);
            frame.render_widget(content, area);
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        // First-match-wins: the most specific context in the stack takes priority.
        // This prevents lower-priority contexts (e.g. Global Quit on Esc) from
        // also firing when a higher-priority context already handled the key.
        for context in self.get_context_stack() {
            if let Some(event) = self.key_event_map.resolve(context, key) {
                self.event_sender.send_app_event(event.clone());
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
                AppFocus::Logs => {
                    stack.push(Logs);
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
