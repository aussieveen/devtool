use ratatui::crossterm::event::Event as CrosstermEvent;
use crate::environment::Environment;
use crate::state::app_state::Tool;
use crate::state::diffchecker::Commit;
use crate::state::focus::AppFocus;
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
    ListMove(ListDir),
    ListSelect(Tool),

    // Tool events
    // DiffChecker
    DiffCheckerListMove(ListDir),
    CommitRefRetrieved(Commit, usize, Environment),
    GenerateDiff,

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