use crate::{
    ui::{layout},
    ui::widgets::*,
    state::app_state::AppState
};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use crate::config::Config;
use crate::events::event::AppEvent::{ListMoveDown, ListMoveUp, ListSelect, Quit, SetFocus};
use crate::events::event::Event;
use crate::events::handler::EventHandler;
use crate::state::app_state::{Focus, Tool};

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    state: AppState,
    events: EventHandler
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config) -> Self {
        Self {
            running: true,
            state: AppState::default(config),
            events: EventHandler::new(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            match self.events.next().await? {
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
                    Quit => self.quit(),
                    SetFocus(focus) => self.set_focus(focus),
                    ListSelect(tool_state) => self.select_tool(tool_state),
                    ListMoveUp => self.list_up(),
                    ListMoveDown => self.list_down()
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
            &self.state,
        );
    }

    /// Handles the key events and updates the state of [`App`].
    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match (&self.state.focus, key.code){
            (Focus::List, KeyCode::Down) => {
                self.events.send(ListMoveDown);
            },
            (Focus::List, KeyCode::Up) => {
                self.events.send(ListMoveUp);
            }
            (Focus::List, KeyCode::Enter) => {
                self.events.send(ListSelect(match self.state.list.state.selected(){
                    Some(1) => Tool::DiffChecker,
                    Some(2) => Tool::TokenGenerator,
                    _ => Tool::Home,
                }))
            }
            (Focus::List, KeyCode::Char('x')) => {
                self.events.send(SetFocus(Focus::Tool))
            }
            (Focus::Tool, KeyCode::Char('x')) => {
                self.events.send(SetFocus(Focus::List))
            }
            (Focus::List, _ ) | ( Focus::Tool, _ ) => {}
        }
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.events.send(Quit),
            // Add other key handlers here.
            _ => {}
        }

        Ok(())
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn set_focus(&mut self, focus: Focus) {
        self.state.focus = focus
    }

    fn select_tool(&mut self, tool: Tool) {
        self.state.tool = tool
    }

    fn list_up(&mut self) {
        self.state.list.state.select_previous();
    }

    fn list_down(&mut self) {
        self.state.list.state.select_next();
    }
}