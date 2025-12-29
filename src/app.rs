use crate::config::Config;
use crate::environment::Environment::{Preproduction, Production};
use crate::events::event::AppEvent::*;
use crate::events::event::{Event, ListDir};
use crate::events::handler::EventHandler;
use crate::events::sender::EventSender;
use crate::state::app::{AppFocus, Tool};
use crate::state::diff_checker::Commit;
use crate::state::token_generator::Focus;
use crate::{state::app::AppState, ui::layout, ui::widgets::*};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};
use std::io::Write;
use std::process::{Command, Stdio};

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
                    ListMove(list_dir) => {
                        let list_state = &mut self.state.tool_list.list_state;
                        Self::update_list(list_state, list_dir);
                        self.event_sender.send(ListSelect(
                            match self.state.tool_list.list_state.selected() {
                                Some(0) => Tool::Home,
                                Some(1) => Tool::DiffChecker,
                                _ => Tool::TokenGenerator,
                            },
                        ));
                    }
                    DiffCheckerListMove(list_dir) => {
                        let list_state = &mut self.state.diff_checker.list_state;
                        Self::update_list(list_state, list_dir);
                    }
                    GenerateDiff => {
                        let service_idx = self.state.diff_checker.list_state.selected().unwrap();

                        if !matches!(
                            self.state.diff_checker.services[service_idx].preprod,
                            Commit::Fetching
                        ) {
                            self.state
                                .diff_checker
                                .set_commit(service_idx, Preproduction)
                                .await
                        }
                        if !matches!(
                            self.state.diff_checker.services[service_idx].prod,
                            Commit::Fetching
                        ) {
                            self.state
                                .diff_checker
                                .set_commit(service_idx, Production)
                                .await
                        }
                    }
                    CommitRefRetrieved(commit, service_idx, env) => match env {
                        Preproduction => {
                            self.state.diff_checker.services[service_idx].preprod = commit
                        }
                        Production => self.state.diff_checker.services[service_idx].prod = commit,
                        _ => {}
                    },
                    TokenGenEnvListMove(list_dir) => {
                        let list_state = &mut self.state.token_generator.env_list_state;
                        Self::update_list(list_state, list_dir);
                    }
                    TokenGenServiceListMove(list_dir) => {
                        let list_state = &mut self.state.token_generator.service_list_state;
                        Self::update_list(list_state, list_dir);
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
                },
            }
        }
        Ok(())
    }

    fn update_list(list_state: &mut ListState, list_dir: ListDir) {
        match list_dir {
            ListDir::Up => list_state.select_previous(),
            ListDir::Down => list_state.select_next(),
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
        match (&self.state.focus, &self.state.current_tool, key.code) {
            // List navigation
            (AppFocus::List, _, KeyCode::Down) => self.event_sender.send(ListMove(ListDir::Down)),
            (AppFocus::List, _, KeyCode::Up) => self.event_sender.send(ListMove(ListDir::Up)),

            // List → Tool focus
            (AppFocus::List, Tool::Home, KeyCode::Right) => {} // no-op
            (AppFocus::List, _, KeyCode::Right) => self.event_sender.send(SetFocus(AppFocus::Tool)),

            // Tool → List focus
            (AppFocus::Tool, Tool::Home | Tool::DiffChecker, KeyCode::Left) => {
                self.event_sender.send(SetFocus(AppFocus::List))
            }

            // DiffChecker key events
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Down) => {
                self.event_sender.send(DiffCheckerListMove(ListDir::Down))
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Up) => {
                self.event_sender.send(DiffCheckerListMove(ListDir::Up))
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Enter) => {
                self.event_sender.send(GenerateDiff)
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Char('o')) => {
                let link = self.state.diff_checker.get_link();
                webbrowser::open(link.as_str()).expect("Failed to open link");
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Char('c')) => {
                let link = self.state.diff_checker.get_link();
                Self::copy_to_clipboard(link).unwrap();
            }

            // TokenGenerator key events
            (AppFocus::Tool, Tool::TokenGenerator, key)
                if matches!(key, KeyCode::Up | KeyCode::Down) =>
            {
                let dir = match key {
                    KeyCode::Up => ListDir::Up,
                    KeyCode::Down => ListDir::Down,
                    _ => unreachable!(),
                };

                let event = match self.state.token_generator.focus {
                    Focus::Service => TokenGenServiceListMove(dir),
                    Focus::Env => TokenGenEnvListMove(dir),
                };

                self.event_sender.send(event);
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Right) => {
                self.event_sender.send(SetTokenGenFocus(Focus::Env));
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Left) => {
                match &self.state.token_generator.focus {
                    Focus::Service => self.event_sender.send(SetFocus(AppFocus::List)),
                    Focus::Env => self.event_sender.send(SetTokenGenFocus(Focus::Service)),
                }
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Enter) => {
                self.event_sender.send(GenerateToken)
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Char('c')) => {
                let token = self
                    .state
                    .token_generator
                    .get_token_for_selected_service_env();
                Self::copy_to_clipboard(token).unwrap();
            }

            // Fallback
            (AppFocus::List, _, _) | (AppFocus::Tool, _, _) => {}
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
