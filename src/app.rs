use std::io::Write;
use std::process::{Command, Stdio};
use crate::{
    ui::{layout},
    ui::widgets::*,
    state::app_state::AppState
};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::ListState;
use crate::config::Config;
use crate::events::event::AppEvent::*;
use crate::events::event::{AppEvent, Event, ListDir};
use crate::events::handler::EventHandler;
use crate::events::sender::EventSender;
use crate::state::app_state::{AppFocus, Tool};
use crate::state::diffchecker::Commit;
use webbrowser;
use crate::environment::Environment::{Preproduction, Production};
use crate::state::token_generator::Focus;

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
            state: AppState::default(config, event_handler.sender()),
            event_handler,
            event_sender,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            match self.event_handler.next().await? {
                Event::Tick => {},
                Event::Crossterm(event) => match event{
                    event::Event::Key(key_event)
                    if key_event.kind == KeyEventKind::Press =>
                        {
                            self.handle_key_events(key_event)?
                        }
                    _ => {}
                },
                Event::App(app_event) => match app_event{
                    Quit => self.running = false,
                    SetFocus(focus) => self.state.focus = focus,
                    ListSelect(tool_state) => self.state.current_tool = tool_state,
                    ListMove(list_dir) => {
                        let list_state = &mut self.state.list.list_state;
                        Self::update_list(list_state, list_dir);
                        self.event_sender.send(ListSelect(match self.state.list.list_state.selected(){
                            Some(1) => Tool::DiffChecker,
                            Some(0) => Tool::Home,
                            _ => Tool::TokenGenerator,
                        }));
                    }
                    DiffCheckerListMove(list_dir) => {
                        let list_state = &mut self.state.diffchecker.list_state;
                        Self::update_list(list_state, list_dir);
                    }
                    GenerateDiff => {
                        let service_idx = self.state.diffchecker.list_state.selected().unwrap();

                        if !matches!(self.state.diffchecker.services[service_idx].preprod,Commit::Fetching) {
                            self.state.diffchecker.set_commit(service_idx, Preproduction).await
                        }
                        if !matches!(self.state.diffchecker.services[service_idx].prod,Commit::Fetching) {
                            self.state.diffchecker.set_commit(service_idx, Production).await
                        }
                    }
                    CommitRefRetrieved(commit, service_idx, env) => {
                        match env {
                            Preproduction => self.state.diffchecker.services[service_idx].preprod = commit,
                            Production => self.state.diffchecker.services[service_idx].prod = commit,
                            _ => {}
                        }
                    },
                    TokenGenEnvListMove(list_dir) => {
                        let list_state = &mut self.state.tokengenerator.env_list_state;
                        Self::update_list(list_state, list_dir);
                    }
                    TokenGenServiceListMove(list_dir ) => {
                        let list_state = &mut self.state.tokengenerator.service_list_state;
                        Self::update_list(list_state, list_dir);
                        self.state.tokengenerator.env_list_state.select_first();
                    }
                    SetTokenGenFocus(focus) => {
                        self.state.tokengenerator.focus = focus;
                    }
                    GenerateToken => {
                        let service_idx = self.state.tokengenerator.service_list_state.selected().unwrap();
                        let env_idx = self.state.tokengenerator.env_list_state.selected().unwrap();

                        self.state.tokengenerator.set_token(service_idx, env_idx).await;
                    }
                },
            }
        }
        Ok(())
    }

    fn update_list(list_state: &mut ListState, list_dir: ListDir){
        match list_dir {
            ListDir::Up => list_state.select_previous(),
            ListDir::Down => list_state.select_next()
        }
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area());

        list::render(
            frame,
            areas.menu,
            &mut self.state,
        );

        tool::render(
            frame,
            areas.content,
            &mut self.state,
        );
    }

    /// Handles the key events and updates the state of [`App`].
    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match (&self.state.focus, &self.state.current_tool, key.code){
            (AppFocus::List, _,  KeyCode::Down) => {
                self.event_sender.send(ListMove(ListDir::Down));
            },
            (AppFocus::List, _, KeyCode::Up) => {
                self.event_sender.send(ListMove(ListDir::Up));
            }
            (AppFocus::List, _, KeyCode::Enter) => {

            }
            (AppFocus::List, _, KeyCode::Char('x') | KeyCode::Right) => {
                self.event_sender.send(SetFocus(AppFocus::Tool))
            }
            (AppFocus::Tool, Tool::Home | Tool::DiffChecker, KeyCode::Char('x') | KeyCode::Left) => {
                self.event_sender.send(SetFocus(AppFocus::List))
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Down) => {
                self.event_sender.send(DiffCheckerListMove(ListDir::Down));
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Up) => {
                self.event_sender.send(DiffCheckerListMove(ListDir::Up));
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Enter) => {
                self.event_sender.send(GenerateDiff)
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Char('o')) => {
                let d = &self.state.diffchecker;
                let link = d.get_link(d.list_state.selected().unwrap());
                webbrowser::open(link.as_str()).expect("Something has gone sideways");
            }
            (AppFocus::Tool, Tool::DiffChecker, KeyCode::Char('c')) => {
                let link = self.state.diffchecker.get_link(self.state.diffchecker.list_state.selected().unwrap());
                Self::copy_to_clipboard(link.as_str()).unwrap();
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Down) => {
                match &self.state.tokengenerator.focus{
                    Focus::Service => self.event_sender.send(TokenGenServiceListMove(ListDir::Down)),
                    Focus::Env => self.event_sender.send(TokenGenEnvListMove(ListDir::Down)),
                };
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Up) => {
                match &self.state.tokengenerator.focus {
                    Focus::Service => self.event_sender.send(TokenGenServiceListMove(ListDir::Up)),
                    Focus::Env => self.event_sender.send(TokenGenEnvListMove(ListDir::Up)),
                }
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Right) => {
                self.event_sender.send(AppEvent::SetTokenGenFocus(Focus::Env));
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Left) => {
                match &self.state.tokengenerator.focus {
                    Focus::Service => self.event_sender.send(SetFocus(AppFocus::List)),
                    Focus::Env => self.event_sender.send(AppEvent::SetTokenGenFocus(Focus::Service))
                }
            }
            (AppFocus::Tool, Tool::TokenGenerator, KeyCode::Enter) => {
                self.event_sender.send(AppEvent::GenerateToken);
            }
            (AppFocus::List, _ , _) | ( AppFocus::Tool, _, _ ) => {}
        }
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.event_sender.send(Quit),
            // Add other key handlers here.
            _ => {}
        }

        Ok(())
    }

    fn copy_to_clipboard(text: &str) -> Result<(), String>{
        if which::which("wl-copy").is_ok() {
            return Self::pipe_to("wl-copy", &[], text)
        }

        if cfg!(target_os = "macos") {
            return Self::pipe_to("pbcopy", &[], text);
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