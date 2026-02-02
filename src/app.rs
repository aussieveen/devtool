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
use std::process::{Command, Stdio, Termination};
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
        match &self.state.focus {
            AppFocus::PopUp => self.handle_popup_events(&self.state.current_tool, key),
            AppFocus::List => self.handle_list_events(&self.state.current_tool, key),
            AppFocus::Tool => self.handle_tool_events(&self.state.current_tool, key)
        }

        self.handle_global_events(self.state.focus, key);

        Ok(())
    }

    fn handle_popup_events(&self, current_tool: &Tool, key: KeyEvent) {
        if Tool::Jira != *current_tool{
            return;
        }

        match key.code {
            k if k.is_backspace() => self.event_sender.send(RemoveTicketIdChar),
            k if k.is_enter() => self.event_sender.send(SubmitTicketId),
            k => {
                if let Some(char) = k.as_char() {
                    self.event_sender.send(AddTicketIdChar(char))
                }
            }
        }
    }

    fn handle_list_events(&self, current_tool: &Tool, key: KeyEvent) {
        if Tool::Home == *current_tool && key.code == KeyCode::Right{
            return
        }

        match key.code{
            KeyCode::Right => self.event_sender.send(SetFocus(AppFocus::Tool)),
            KeyCode::Down => self.event_sender.send(ListMove(Direction::Down)),
            KeyCode::Up=> self.event_sender.send(ListMove(Direction::Up)),
            _ => {}
        }
    }

    fn handle_tool_events(&self, current_tool: &Tool, key: KeyEvent) {
        if matches![current_tool, Tool::Home | Tool::ServiceStatus | Tool::Jira]
            && key.code == KeyCode::Left {
            self.event_sender.send(SetFocus(AppFocus::List))
        }

        match (current_tool) {
            Tool::Home => {},
            Tool::ServiceStatus => self.handle_service_status_key_events(key),
            Tool::TokenGenerator => self.handle_token_generator_key_events(key),
            Tool::Jira => self.handle_jira_key_events(key)
        }
    }


    fn handle_service_status_key_events(&self, key: KeyEvent) {
        match key.code {
            KeyCode::Down => self.event_sender.send(ServiceStatusListMove(Direction::Down)),
            KeyCode::Up => self.event_sender.send(ServiceStatusListMove(Direction::Up)),
            KeyCode::Char('o') => {
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
            KeyCode::Char('c') => {
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
            KeyCode::Char('s') => {
                self.event_sender.send(ScanServices)
            }
            _ => {}
        }
    }

    fn handle_token_generator_key_events(&self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Up | KeyCode::Down) {
            let dir = match key.code {
                KeyCode::Up => Direction::Up,
                KeyCode::Down => Direction::Down,
                _ => unreachable!(),
            };

            let event = match self.state.token_generator.focus {
                Focus::Service => TokenGenServiceListMove(dir),
                Focus::Env => TokenGenEnvListMove(dir),
            };

            self.event_sender.send(event);
            return
        }

        match key.code {
            KeyCode::Right => self.event_sender.send(SetTokenGenFocus(Focus::Env)),
            KeyCode::Left => match self.state.token_generator.focus {
                Focus::Service => self.event_sender.send(SetFocus(AppFocus::List)),
                Focus::Env => self.event_sender.send(SetTokenGenFocus(Focus::Service)),
            },
            KeyCode::Enter => self.event_sender.send(GenerateToken),
            KeyCode::Char('c') => {
                if let Some(token) = self
                    .state
                    .token_generator
                    .get_token_for_selected_service_env()
                    .value()
                {
                    let _result = string_copy::copy_to_clipboard(token.to_string());
                    todo!("Display errors as pop up somehow");
                };
            },
            _ => {}
        }
    }

    fn handle_jira_key_events(&self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::SHIFT, KeyCode::Down) => self.event_sender.send(JiraTicketMove(Direction::Down)),
            (KeyModifiers::SHIFT, KeyCode::Up) => self.event_sender.send(JiraTicketMove(Direction::Up)),
            (_, KeyCode::Down) => self.event_sender.send(JiraTicketListMove(Direction::Down)),
            (_, KeyCode::Up) => self.event_sender.send(JiraTicketListMove(Direction::Up)),
            (_, KeyCode::Char('a')) => self.event_sender.send(NewJiraTicketPopUp),
            (_, KeyCode::Char('x')) => self.event_sender.send(RemoveTicket),
            _ => {}
        }
    }

    fn handle_global_events(&self, focus: AppFocus, key: KeyEvent) {
        // Global quit
        if (matches!(key.code, KeyCode::Char('q')) && !matches!(self.state.focus, AppFocus::PopUp))
            || matches!(key.code, KeyCode::Esc)
        {
            self.event_sender.send(Quit);
        }
    }
}
