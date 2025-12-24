use crate::{
    ui::{layout},
    ui::widgets::*,
    state::app_state::AppState
};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use crate::config::Config;
use crate::events::event::AppEvent::*;
use crate::events::event::Event;
use crate::events::handler::EventHandler;
use crate::events::sender::EventSender;
use crate::state::app_state::{Focus, Tool};
use crate::state::diffchecker::Commit;

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    state: AppState,
    event_handler: EventHandler,
    event_sender: EventSender
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config) -> Self {
        let event_handler = EventHandler::new();
        let event_sender = event_handler.sender();
        Self {
            running: true,
            state: AppState::default(config),
            event_handler,
            event_sender
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
                    ListMoveUp => self.state.list.state.select_previous(),
                    ListMoveDown => self.state.list.state.select_next(),
                    DiffCheckerListMoveDown => self.state.diffchecker.state.select_next(),
                    DiffCheckerListMoveUp => self.state.diffchecker.state.select_previous(),
                    GenerateDiff => {
                        let service_idx = self.state.diffchecker.state.selected().unwrap();

                        if !matches!(self.state.diffchecker.services[service_idx].preprod,Commit::Fetching) {
                            self.state.diffchecker.set_preprod_commit(service_idx)
                        }
                        if !matches!(self.state.diffchecker.services[service_idx].prod,Commit::Fetching) {
                            self.state.diffchecker.set_prod_commit(service_idx)
                        }
                    }
                },
            }
        }
        Ok(())
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
            (Focus::List, _,  KeyCode::Down) => {
                self.event_sender.send(ListMoveDown);
            },
            (Focus::List, _, KeyCode::Up) => {
                self.event_sender.send(ListMoveUp);
            }
            (Focus::List, _, KeyCode::Enter) => {
                self.event_sender.send(ListSelect(match self.state.list.state.selected(){
                    Some(1) => Tool::DiffChecker,
                    Some(2) => Tool::TokenGenerator,
                    _ => Tool::Home,
                }))
            }
            (Focus::List, _, KeyCode::Char('x')) => {
                self.event_sender.send(SetFocus(Focus::Tool))
            }
            (Focus::Tool, _, KeyCode::Char('x')) => {
                self.event_sender.send(SetFocus(Focus::List))
            }
            (Focus::Tool, Tool::DiffChecker, KeyCode::Down) => {
                self.event_sender.send(DiffCheckerListMoveDown);
            }
            (Focus::Tool, Tool::DiffChecker, KeyCode::Up) => {
                self.event_sender.send(DiffCheckerListMoveUp);
            }
            (Focus::Tool, Tool::DiffChecker, KeyCode::Enter) => {
                self.event_sender.send(GenerateDiff)
            }
            (Focus::List, _ , _) | ( Focus::Tool, _, _ ) => {}
        }
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.event_sender.send(Quit),
            // Add other key handlers here.
            _ => {}
        }

        Ok(())
    }
}