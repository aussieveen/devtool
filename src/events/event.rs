use ratatui::crossterm::event::Event as CrosstermEvent;
use crate::environment::Environment;
use crate::state::app::{AppFocus, Tool};
use crate::state::diff_checker::Commit;
use crate::state::token_generator::{Focus, Token};

#[derive(Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
}

#[derive(Debug)]
pub enum AppEvent {
    // List events
    ListMove(ListDir), // Move up and down tool List
    ListSelect(Tool), // Select item on tool list

    // Tool events
    // DiffChecker
    DiffCheckerListMove(ListDir), // Move up and down on service list
    // Commit reference received for service on env
    // usize is the index of the service in the ListState
    CommitRefRetrieved(Commit, usize, Environment), 
    GenerateDiff, // Generate diff url

    // TokenGenerator
    TokenGenServiceListMove(ListDir),
    TokenGenEnvListMove(ListDir),
    SetTokenGenFocus(Focus),
    GenerateToken,
    TokenGenerated(Token, usize, usize),

    // Generic Events
    SetFocus(AppFocus),
    Quit,
}

#[derive(Debug)]
pub enum ListDir {
    Up,
    Down
}