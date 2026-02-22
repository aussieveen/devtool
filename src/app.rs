use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
use crate::client::auth_zero::api::{AuthZeroApi, ImmediateAuthZeroApi};
use crate::client::healthcheck::api::{HealthcheckApi, ImmediateHealthcheckApi};
use crate::client::jira::api::{ImmediateJiraApi, JiraApi};
use crate::config::model::Config;
use crate::events::event::AppEvent::*;
use crate::events::event::{AppEvent, Event};
use crate::events::handler::EventHandler;
use crate::events::sender::EventSender;
use crate::events::tools::{jira, service_status, token_generator};
use crate::input::key_bindings::register_bindings;
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{
    ErrorPopUp, Global, List, Popup, TokenGen, ToolIgnore,
};
use crate::input::key_event_map::KeyEventMap;
pub(crate) use crate::state::app::{AppFocus, Tool};
use crate::utils::popup::popup_area;
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
    key_event_map: KeyEventMap,

    // External Services
    pub(crate) jira_api: Arc<dyn JiraApi>,
    pub(crate) auth_zero_api: Arc<dyn AuthZeroApi>,
    pub(crate) healthcheck_api: Arc<dyn HealthcheckApi>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config) -> Self {
        let event_handler = EventHandler::new();
        let event_sender = event_handler.sender();
        Self {
            running: true,
            state: AppState::new(&config),
            event_handler,
            event_sender,
            config,
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
            DismissPopup => self.state.error = None,

            CopyToClipboard => match self.state.current_tool {
                ServiceStatus => service_status::handle_event(self, CopyToClipboard),
                TokenGenerator => token_generator::handle_event(self, CopyToClipboard),
                _ => {}
            },
            OpenInBrowser => {
                if self.state.current_tool == ServiceStatus {
                    service_status::handle_event(self, OpenInBrowser)
                }
            }

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
            | e @ NewJiraTicketPopUp
            | e @ AddTicketIdChar(..)
            | e @ RemoveTicketIdChar
            | e @ SubmitTicketId
            | e @ TicketRetrieved(..)
            | e @ RemoveTicket
            | e @ JiraTicketMove(..)
            | e @ JiraTicketListUpdate => jira::handle_event(self, e),
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area());

        list::render(frame, areas.menu, &mut self.state);

        tool::render(frame, areas.content, &mut self.state, &self.config);

        footer::render(frame, areas.footer);

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

            let area = popup_area(frame.area(), 50, 7);
            frame.render_widget(Clear, area);
            frame.render_widget(paragraph, area);
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        for context in self.get_context_stack() {
            if let Some(event) = self.key_event_map.resolve(context, key) {
                self.event_sender.send(event.clone());
            }
        }

        Ok(())
    }

    fn get_context_stack(&self) -> Vec<KeyContext> {
        let mut stack = Vec::new();

        // If an Error pop up is displayed, don't allow additional contexts i.e
        // disable all key contexts except global and the error popup.
        if self.state.error.is_some() {
            stack.push(ErrorPopUp);
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
                AppFocus::PopUp => {
                    stack.push(Popup(Jira));
                }
            }
        }

        stack.push(Global);
        stack
    }
}
