use crate::config::{Config};
use crate::events::event::AppEvent::*;
use crate::events::event::{AppEvent, Direction, Event};
use crate::events::handler::EventHandler;
use crate::events::sender::EventSender;
use crate::state::app::{AppFocus, Tool};
use crate::state::token_generator::Focus;
use crate::{state::app::AppState, ui::layout, ui::widgets::*};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use crate::events::tools::{service_status, token_generator, jira};
use crate::utils::{browser, update_list_state, string_copy};

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    pub state: AppState,
    event_handler: EventHandler,
    pub event_sender: EventSender,
    pub config: Config,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config) -> Self {
        let event_handler = EventHandler::new();
        let event_sender = event_handler.sender();
        Self {
            running: true,
            state: AppState::new(&config, event_handler.sender()),
            event_handler,
            event_sender,
            config,
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

    fn handle_app_event(&mut self, app_event: AppEvent){
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
            // service status events
            e @ ServiceStatusListMove(..)
            | e @ ScanServices
            | e @ ScanServiceEnv(..)
            | e @ GetCommitRefOk(..)
            | e @ GetCommitRefErrored(..) => {
                service_status::handle_event(self, e)
            }
            // token generator events
            e @ TokenGenEnvListMove(..)
            | e @ TokenGenServiceListMove(..)
            | e @ SetTokenGenFocus(..)
            | e @ GenerateToken
            | e @ TokenGenerated(..)
            | e @ TokenFailed(..) => {
                token_generator::handle_event(self, e)
            }
            // jira ticket events
            e @ JiraTicketListMove(..)
            | e @ NewJiraTicketPopUp
            | e @ AddTicketIdChar(..)
            | e @ RemoveTicketIdChar
            | e @ SubmitTicketId
            | e @ TicketRetrieved(..)
            | e @ RemoveTicket
            | e @ JiraTicketMove(..) => {
                jira::handle_event(self, e)
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area());

        list::render(frame, areas.menu, &mut self.state);

        tool::render(frame, areas.content, &mut self.state, &self.config);

        footer::render(frame, areas.footer)
    }

    /// Handles the key events and updates the state of [`App`].
    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match (
            &self.state.focus,
            &self.state.current_tool,
            key.code,
            key.modifiers,
        ) {
            // List navigation
            (AppFocus::List, _, KeyCode::Down, _) => {
                self.event_sender.send(ListMove(Direction::Down))
            }
            (AppFocus::List, _, KeyCode::Up, _) => self.event_sender.send(ListMove(Direction::Up)),

            // List → Tool focus
            (AppFocus::List, Tool::Home, KeyCode::Right, _) => {} // no-op
            (AppFocus::List, _, KeyCode::Right, _) => {
                self.event_sender.send(SetFocus(AppFocus::Tool))
            }

            // Tool → List focus
            (AppFocus::Tool, Tool::Home | Tool::ServiceStatus | Tool::Jira, KeyCode::Left, _) => {
                self.event_sender.send(SetFocus(AppFocus::List))
            }

            // ServiceStatus key events
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Down, _) => self
                .event_sender
                .send(ServiceStatusListMove(Direction::Down)),
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Up, _) => {
                self.event_sender.send(ServiceStatusListMove(Direction::Up))
            }
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Char('o'), _) => {
                if self.state.service_status.has_link()
                    && let Some(service_idx) = self.state.service_status.get_selected_service_idx()
                {
                    let link = self
                        .state
                        .service_status
                        .get_link(&self.config.servicestatus[service_idx].repo);
                    browser::open_link_in_browser(link)
                }
            }
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Char('c'), _) => {
                if self.state.service_status.has_link()
                    && let Some(service_idx) = self.state.service_status.get_selected_service_idx()
                {
                    let link = self
                        .state
                        .service_status
                        .get_link(&self.config.servicestatus[service_idx].repo);
                    string_copy::copy_to_clipboard(link).unwrap();
                }
            }
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Char('s'), _) => {
                self.event_sender.send(ScanServices)
            }
            // TokenGenerator key events
            (AppFocus::Tool, Tool::TokenGenerator, key, _)
                if matches!(key, KeyCode::Up | KeyCode::Down) =>
            {
                let dir = match key {
                    KeyCode::Up => Direction::Up,
                    KeyCode::Down => Direction::Down,
                    _ => unreachable!(),
                };

                let event = match self.state.token_generator.focus {
                    Focus::Service => TokenGenServiceListMove(dir),
                    Focus::Env => TokenGenEnvListMove(dir),
                };

                self.event_sender.send(event);
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Right, _) => {
                self.event_sender.send(SetTokenGenFocus(Focus::Env));
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Left, _) => {
                match &self.state.token_generator.focus {
                    Focus::Service => self.event_sender.send(SetFocus(AppFocus::List)),
                    Focus::Env => self.event_sender.send(SetTokenGenFocus(Focus::Service)),
                }
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Enter, _) => {
                self.event_sender.send(GenerateToken)
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Char('c'), _) => {
                if let Some(token) = self
                    .state
                    .token_generator
                    .get_token_for_selected_service_env()
                    .value()
                {
                    let _result = string_copy::copy_to_clipboard(token.to_string());
                    todo!("Display errors as pop up somehow");
                };
            }
            (AppFocus::Tool, Tool::Jira, key, KeyModifiers::SHIFT) => {
                if let Some(direction) = match key {
                    KeyCode::Down => Some(Direction::Down),
                    KeyCode::Up => Some(Direction::Up),
                    _ => None,
                } {
                    self.event_sender.send(JiraTicketMove(direction));
                }
            }
            (AppFocus::Tool, Tool::Jira, KeyCode::Down, _) => {
                self.event_sender.send(JiraTicketListMove(Direction::Down))
            }
            (AppFocus::Tool, Tool::Jira, KeyCode::Up, _) => {
                self.event_sender.send(JiraTicketListMove(Direction::Up))
            }
            (AppFocus::Tool, Tool::Jira, KeyCode::Char('a'), _) => {
                self.event_sender.send(NewJiraTicketPopUp)
            }
            (AppFocus::Tool, Tool::Jira, KeyCode::Char('x'), _) => {
                self.event_sender.send(RemoveTicket)
            }
            (AppFocus::PopUp, Tool::Jira, key_code, _) => {
                if key_code.is_backspace() {
                    self.event_sender.send(RemoveTicketIdChar);
                } else if key_code.is_enter() {
                    self.event_sender.send(SubmitTicketId);
                } else if let Some(char) = key_code.as_char() {
                    self.event_sender.send(AddTicketIdChar(char))
                }
            }

            // Fallback
            (AppFocus::List, _, _, _) | (AppFocus::Tool, _, _, _) | (AppFocus::PopUp, _, _, _) => {}
        }

        // Global quit
        if matches!(key.code, KeyCode::Esc)
            || (matches!(key.code, KeyCode::Char('q'))
                && !matches!(self.state.focus, AppFocus::PopUp))
        {
            self.event_sender.send(Quit);
        }

        Ok(())
    }
}
