use crate::{
    ui::{layout},
    ui::widgets::*,
    state::state::AppState
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use crate::state::state::{Focus, ToolState};

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    state: AppState
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            state: AppState::default()
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
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

        tools::render(
            frame,
            areas.menu,
            &mut self.state,
        );

        content::render(
            frame,
            areas.content,
            &self.state,
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (&self.state.focus, key.code){
            (Focus::Menu, KeyCode::Down) => {
                self.state.list.state.select_next();
            },
            (Focus::Menu, KeyCode::Up) => {
                self.state.list.state.select_previous();
            }
            (Focus::Menu, KeyCode::Enter) => {
                self.state.tool = match self.state.list.state.selected(){
                    Some(1) => ToolState::TokenGenerator,
                    Some(2) => ToolState::DiffChecker,
                    _ => ToolState::Home,
                };
            }
            (Focus::Menu, KeyCode::Char('x')) => {
                self.state.focus = Focus::Content
            }
            (Focus::Content, KeyCode::Char('x')) => {
                self.state.focus = Focus::Menu
            }
            (Focus::Menu, _ ) | ( Focus::Content, _ ) => {}
        }
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}