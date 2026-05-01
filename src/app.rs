use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
use crate::client::auth_zero::api::{AuthZeroApi, ImmediateAuthZeroApi};
use crate::client::healthcheck::api::{HealthcheckApi, ImmediateHealthcheckApi};
use crate::client::jira::api::{ImmediateJiraApi, JiraApi};
use crate::config::loader::ConfigLoader;
use crate::config::model::Config;
use crate::event::events::AppEvent::*;
use crate::event::events::GenericEvent::{
    CopyToClipboard, OpenInBrowser, Quit, QuitConfirm, SetFocus,
};
use crate::event::events::JiraEvent::ScanTickets;
use crate::event::events::ServiceStatusEvent::Scan;
use crate::event::events::{AppEvent, Event, GenericEvent};
use crate::event::handler::EventHandler;
use crate::event::handlers::config::{
    jira as jira_config, service_status as service_status_config,
    token_generator as token_generator_config,
};
use crate::event::handlers::tools::{jira, service_status, token_generator};
use crate::event::sender::EventSender;
use crate::input::key_bindings::register_bindings;
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{
    Editing, Global, List, Logs, Popup as PopupCtx, TokenGen, ToolConfigEditing, ToolIgnore,
};
use crate::input::key_event_map::KeyEventMap;
use crate::popup::model::Popup;
pub(crate) use crate::state::app::{AppFocus, Tool};
use crate::state::log::{LogEntry, LogLevel, log_source};
use crate::ui::widgets::popup::{Part, Type};
use crate::utils::update_list_state;
use crate::{state::app::AppState, ui::layout, ui::widgets::*};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
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
        self.state.log.push_log(LogEntry::new(
            LogLevel::Info,
            log_source::APP,
            "App started — config loaded",
        ));

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
                Event::ServiceStatus(event) => service_status::handle_event(&mut self, event),
                Event::ServiceStatusConfig(event) => {
                    service_status_config::handle_event(&mut self, event)
                }
                Event::TokenGenerator(event) => token_generator::handle_event(&mut self, event),
                Event::TokenGeneratorConfig(event) => {
                    token_generator_config::handle_event(&mut self, event)
                }
                Event::Jira(event) => jira::handle_event(&mut self, event),
                Event::JiraConfig(event) => jira_config::handle_event(&mut self, event),
                Event::Generic(event) => self.handle_generic_event(event),
            }
        }
        Ok(())
    }

    fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            OpenLogs => {
                self.state.focus = AppFocus::Logs;
                if self.state.has_popup() {
                    self.event_sender.send_app_event(DismissPopup);
                    self.state.log.select_logs()
                } else {
                    if self.state.log.selected_item == crate::state::log::LogsItem::Activity {
                        self.state.log.mark_activity_seen();
                    }
                }
            }
            LogsListMove(direction) => {
                use crate::event::events::Direction;
                match direction {
                    Direction::Down => {
                        self.state.log.select_logs();
                    }
                    Direction::Up => {
                        self.state.log.select_activity();
                        if self.state.log.selected_item == crate::state::log::LogsItem::Activity {
                            self.state.log.mark_activity_seen();
                        }
                    }
                }
            }
            ActivityEvent(source, message) => {
                self.state.log.push_activity(source, message);
            }
            AppLog(entry) => {
                let title = entry.title.clone();
                let level = entry.level;
                self.state.log.push_log(entry);
                if level <= LogLevel::Error {
                    self.state.popup = Some(Popup::new(
                        Type::Error,
                        title,
                        vec![
                            Part::Text("See "),
                            Part::Key("3"),
                            Part::Text(" Logs for details  "),
                        ],
                    ));
                }
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
            DismissPopup => self.state.popup = None,

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
                let ss_form_open = self.state.service_status_config_editor.has_open_form();
                let tg_form_open = self.state.token_generator_config_editor.has_open_form();
                let jira_form_open = self.state.jira_config_editor.has_open_form();
                if ss_form_open {
                    self.state.service_status_config_editor.close_form();
                } else if tg_form_open {
                    self.state.token_generator_config_editor.close_form();
                } else if jira_form_open {
                    self.state.jira_config_editor.close_form();
                } else {
                    self.state.focus = AppFocus::Config;
                }
            }
        }
    }

    fn handle_generic_event(&mut self, event: GenericEvent) {
        match event {
            Quit => {
                self.state.popup = Some(
                    Popup::new(
                        Type::Confirm,
                        "Confirm Quit".to_string(),
                        vec![Part::Key("q"), Part::Text(" again to quit  ")],
                    )
                    .with_action('q', "quit", Event::Generic(QuitConfirm)),
                )
            }
            QuitConfirm => self.running = false,
            SetFocus(focus) => self.state.focus = focus,
            CopyToClipboard => match self.state.current_tool {
                ServiceStatus => service_status::handle_generic_event(self, CopyToClipboard),
                TokenGenerator => token_generator::handle_generic_event(self, CopyToClipboard),
                _ => {}
            },
            OpenInBrowser => match self.state.current_tool {
                ServiceStatus => service_status::handle_generic_event(self, OpenInBrowser),
                Jira => jira::handle_generic_event(self, OpenInBrowser),
                _ => {}
            },
        }
    }
    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area(), self.state.effective_focus());

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

        if let Some(popup) = &self.state.popup {
            popup::render(frame, popup)
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        if self.state.has_popup() {
            if let KeyCode::Char(c) = key.code
                && let Some(popup) = &self.state.popup
                && let Some(action) = popup.actions.iter().find(|a| a.key == c)
            {
                let event = action.event.clone();
                self.event_sender.send_event(event);
            }
            self.state.popup = None;
            return Ok(());
        }
        // First-match-wins: the most specific context in the stack takes priority.
        // This prevents lower-priority contexts (e.g. Global Quit on Esc) from
        // also firing when a higher-priority context already handled the key.
        for context in self.get_context_stack() {
            if let Some(event) = self.key_event_map.resolve(context, key) {
                self.event_sender.send_event(event.clone());
                break;
            }
        }

        Ok(())
    }

    fn get_context_stack(&mut self) -> Vec<KeyContext> {
        let mut stack = Vec::new();

        // If a popup is displayed, don't allow additional contexts i.e
        // disable all key contexts except global and the popup dismiss binding.
        if self.state.has_popup() {
            stack.push(PopupCtx);
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
                        && self.state.service_status_config_editor.has_open_form()
                    {
                        stack.push(Editing(ServiceStatus));
                    } else if tool == TokenGenerator
                        && self.state.token_generator_config_editor.has_open_form()
                    {
                        stack.push(Editing(TokenGenerator));
                    } else if tool == Jira && self.state.jira_config_editor.has_open_form() {
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
