use crate::config::Config;
use crate::environment::Environment::{Preproduction, Production, Staging};
use crate::events::event::AppEvent::*;
use crate::events::event::{Event, Direction};
use crate::events::handler::EventHandler;
use crate::events::sender::EventSender;
use crate::state::app::{AppFocus, Tool};
use crate::state::service_status::Commit;
use crate::state::token_generator::Focus;
use crate::{state::app::AppState, ui::layout, ui::widgets::*};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    state: AppState,
    event_handler: EventHandler,
    event_sender: EventSender,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config) -> Self {
        let event_handler = EventHandler::new();
        let event_sender = event_handler.sender();
        Self {
            running: true,
            state: AppState::new(config, event_handler.sender()),
            event_handler,
            event_sender,
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
                Event::App(app_event) => match app_event {
                    Quit => self.running = false,
                    SetFocus(focus) => self.state.focus = focus,
                    ListSelect(tool_state) => self.state.current_tool = tool_state,
                    ListMove(direction) => {
                        let tool_list = &mut self.state.tool_list;
                        Self::update_list(&mut tool_list.list_state, direction, tool_list.items.len());
                        if let Some(index) = tool_list.list_state.selected(){
                            if let Some(tool) = tool_list.items.get(index).cloned(){
                                self.event_sender.send(ListSelect(tool))
                            }
                        }
                    }
                    ServiceStatusListMove(direction) => {
                        let list_state = &mut self.state.service_status.list_state;
                        let list_limit = self.state.service_status.services.len();
                        Self::update_noneable_list(list_state, direction, list_limit);
                    }
                    ScanServices => {
                        let len = self.state.service_status.services.len();

                        for service_idx in 0..len {
                            if !matches!(
                                self.state.service_status.services[service_idx].staging,
                                Commit::Fetching
                            ) {
                                self.state
                                    .service_status
                                    .set_commit(service_idx, Staging)
                                    .await
                            }
                            if !matches!(
                                self.state.service_status.services[service_idx].preprod,
                                Commit::Fetching
                            ) {
                                self.state
                                    .service_status
                                    .set_commit(service_idx, Preproduction)
                                    .await
                            }
                            if !matches!(
                                self.state.service_status.services[service_idx].prod,
                                Commit::Fetching
                            ) {
                                self.state
                                    .service_status
                                    .set_commit(service_idx, Production)
                                    .await
                            }
                        }
                    }
                    GenerateDiff => {
                        let service_idx = self.state.service_status.list_state.selected().unwrap();
                        if !matches!(
                            self.state.service_status.services[service_idx].staging,
                            Commit::Fetching
                        ) {
                            self.state
                                .service_status
                                .set_commit(service_idx, Staging)
                                .await
                        }
                        if !matches!(
                            self.state.service_status.services[service_idx].preprod,
                            Commit::Fetching
                        ) {
                            self.state
                                .service_status
                                .set_commit(service_idx, Preproduction)
                                .await
                        }
                        if !matches!(
                            self.state.service_status.services[service_idx].prod,
                            Commit::Fetching
                        ) {
                            self.state
                                .service_status
                                .set_commit(service_idx, Production)
                                .await
                        }
                    }
                    CommitRefRetrieved(commit, service_idx, env) => match env {
                        Staging => self.state.service_status.services[service_idx].staging = commit,
                        Preproduction => {
                            self.state.service_status.services[service_idx].preprod = commit
                        }
                        Production => self.state.service_status.services[service_idx].prod = commit,
                        _ => {}
                    },
                    TokenGenEnvListMove(direction) => {
                        let list_state = &mut self.state.token_generator.env_list_state;
                        let selected_service = self.state.token_generator.service_list_state.selected().unwrap_or(0);
                        Self::update_list(list_state, direction, self.state.token_generator.services[selected_service].tokens.len());
                    }
                    TokenGenServiceListMove(direction) => {
                        let list_state = &mut self.state.token_generator.service_list_state;
                        Self::update_list(list_state, direction, self.state.token_generator.services.len());
                        self.state.token_generator.env_list_state.select_first();
                    }
                    SetTokenGenFocus(focus) => {
                        self.state.token_generator.focus = focus;
                    }
                    GenerateToken => {
                        let service_idx = self
                            .state
                            .token_generator
                            .service_list_state
                            .selected()
                            .unwrap();
                        let env_idx = self
                            .state
                            .token_generator
                            .env_list_state
                            .selected()
                            .unwrap();

                        self.state
                            .token_generator
                            .set_token(service_idx, env_idx)
                            .await;
                    }
                    TokenGenerated(token, service_idx, env_idx) => {
                        let service = &mut self.state.token_generator.services[service_idx];
                        let credentials = &service.credentials[env_idx];

                        service.tokens.insert(credentials.env.clone(), token);
                    }
                    JiraTicketListMove(direction) => {
                        if(self.state.jira.is_some()) {
                            let list_len = self.state.jira.clone().unwrap().tickets.len();
                            let list_state = &mut self.state.jira.as_mut().unwrap().list_state;
                            Self::update_noneable_list(list_state, direction, list_len);
                        }
                    }
                    JiraTicketMove(direction) => {}
                },
            }
        }
        Ok(())
    }

    fn update_list(list_state: &mut ListState, direction: Direction, len: usize) {
        match direction {
            Direction::Up => list_state.select_previous(),
            Direction::Down => Self::select_next(list_state, len)
        }
    }

    fn update_noneable_list(list_state: &mut ListState, direction: Direction, len: usize) {
        let selected = list_state.selected();
        match direction {
            Direction::Up => {
                if selected.unwrap_or(0) > 0 {
                    list_state.select_previous();
                } else {
                    list_state.select(None);
                }
            }
            Direction::Down => Self::select_next(list_state, len)
        }
    }

    fn select_next(list_state: &mut ListState, len: usize){
        let selected = list_state.selected().unwrap_or(0);
        let n = len.saturating_sub(1);
        if selected == n {
            list_state.select(Some(n));
        } else {
            list_state.select_next();
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area());

        list::render(frame, areas.menu, &mut self.state);

        tool::render(frame, areas.content, &mut self.state);

        footer::render(frame, areas.footer)
    }

    /// Handles the key events and updates the state of [`App`].
    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match (&self.state.focus, &self.state.current_tool, key.code, key.modifiers) {
            // List navigation
            (AppFocus::List, _, KeyCode::Down, _) => self.event_sender.send(ListMove(Direction::Down)),
            (AppFocus::List, _, KeyCode::Up, _) => self.event_sender.send(ListMove(Direction::Up)),

            // List → Tool focus
            (AppFocus::List, Tool::Home, KeyCode::Right, _) => {} // no-op
            (AppFocus::List, _, KeyCode::Right, _) => self.event_sender.send(SetFocus(AppFocus::Tool)),

            // Tool → List focus
            (AppFocus::Tool, Tool::Home | Tool::ServiceStatus | Tool::Jira, KeyCode::Left, _) => {
                self.event_sender.send(SetFocus(AppFocus::List))
            }

            // ServiceStatus key events
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Down, _) => {
                self.event_sender.send(ServiceStatusListMove(Direction::Down))
            }
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Up, _) => {
                self.event_sender.send(ServiceStatusListMove(Direction::Up))
            }
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Enter, _) => {
                self.event_sender.send(GenerateDiff)
            }
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Char('o'), _) => {
                if self.state.service_status.has_link() {
                    let link = self.state.service_status.get_link();
                    webbrowser::open(link.as_str()).expect("Failed to open link");
                }
            }
            (AppFocus::Tool, Tool::ServiceStatus, KeyCode::Char('c'), _) => {
                if self.state.service_status.has_link() {
                    let link = self.state.service_status.get_link();
                    Self::copy_to_clipboard(link).unwrap();
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
                let token = self
                    .state
                    .token_generator
                    .get_token_for_selected_service_env();
                Self::copy_to_clipboard(token).unwrap();
            }
            (AppFocus::Tool, Tool::Jira, KeyCode::Down, _) => {
                self.event_sender.send(JiraTicketListMove(Direction::Down))
            }
            (AppFocus::Tool, Tool::Jira, KeyCode::Up, _) => {
                self.event_sender.send(JiraTicketListMove(Direction::Up))
            }

            // Fallback
            (AppFocus::List, _, _, _) | (AppFocus::Tool, _, _, _) => {}
        }

        // Global quit
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            self.event_sender.send(Quit);
        }

        Ok(())
    }

    fn copy_to_clipboard(text: String) -> Result<(), String> {
        let str = text.as_str();
        if which::which("wl-copy").is_ok() {
            return Self::pipe_to("wl-copy", &[], str);
        }

        if cfg!(target_os = "macos") {
            return Self::pipe_to("pbcopy", &[], str);
        }

        Ok(())
    }

    fn pipe_to(cmd: &str, args: &[&str], text: &str) -> Result<(), String> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to run {cmd}: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            // Ignore broken pipe — clipboard tool may exit early
            let _ = stdin.write_all(text.as_bytes());
        }

        let _ = child.wait();
        Ok(())
    }
}
