use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    widgets::{Block, Paragraph},
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Borders, List, ListItem};
use crate::state::{BlockState, ContentState, State};

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    state: State
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            state: State::default()
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
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Min(30),
                Constraint::Percentage(90)
            ])
            .split(frame.area());

        let highlighted_style = Style::default().fg(Color::Green);
        let default_style = Style::default();

        let menu_style = match self.state.block {
            BlockState::Menu => highlighted_style,
            _ => default_style
        };
        let content_style = match self.state.block {
            BlockState::Content => highlighted_style,
            _ => default_style
        };

        let content = match self.state.content{
            ContentState::TokenGenerator => "This is the token generator",
            ContentState::DiffChecker => "This is the diff checker",
            _ => "This is the home page",
        };

        let list = List::new(self.state.menu.items.iter().map(|i| ListItem::new(*i)))
            .block(Block::bordered().border_style(menu_style).title("Menu"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true);
        frame.render_stateful_widget(
            list,
            layout[0],
            &mut self.state.menu.state
        );

        let paragraph = Paragraph::new(content)
            .block(Block::new().title("TITLE").borders(Borders::ALL).border_style(content_style));
        frame.render_widget(
            paragraph,
            layout[1]
        )
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
        match (&self.state.block, key.code){
            (BlockState::Menu, KeyCode::Down) => {
                self.state.menu.state.select_next();
            },
            (BlockState::Menu, KeyCode::Up) => {
                self.state.menu.state.select_previous();
            }
            (BlockState::Menu, KeyCode::Enter) => {
                self.state.content = match self.state.menu.state.selected(){
                    Some(1) => ContentState::TokenGenerator,
                    Some(2) => ContentState::DiffChecker,
                    _ => ContentState::Home,
                };
                self.state.block = BlockState::Content
            }
            (BlockState::Menu, KeyCode::Right) => {
                self.state.block = BlockState::Content
            }
            (BlockState::Content, KeyCode::Left) => {
                self.state.block = BlockState::Menu
            }
            (BlockState::Menu, _ ) | ( BlockState::Content, _ ) => {}
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