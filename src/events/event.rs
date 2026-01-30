use crate::environment::Environment;
use crate::state::app::{AppFocus, Tool};
use crate::state::service_status::Commit;
use crate::state::token_generator::{Focus, Token};
use ratatui::crossterm::event::Event as CrosstermEvent;

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
    ListMove(Direction), // Move up and down tool List
    ListSelect(Tool),  // Select item on tool list

    // Tool events
    // ServiceStatus
    ServiceStatusListMove(Direction), // Move up and down on service list
    // Commit reference received for service on env
    // usize is the index of the service in the ListState
    CommitRefRetrieved(Commit, usize, Environment),
    GenerateDiff, // Generate diff url
    ScanServices, // Scan all services

    // TokenGenerator
    TokenGenServiceListMove(Direction),
    TokenGenEnvListMove(Direction),
    SetTokenGenFocus(Focus),
    GenerateToken,
    TokenGenerated(Token, usize, usize),
    
    // Jira
    JiraTicketListMove(Direction),
    JiraTicketMove(Direction),
    NewJiraTicketPopUp,
    AddTicketIdChar(char),
    RemoveTicketIdChar,
    SubmitTicketId,
    RemoveTicket,

    // Generic Events
    SetFocus(AppFocus),
    Quit,
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
}
