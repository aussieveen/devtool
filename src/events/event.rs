use crate::client::jira::models::TicketResponse;
use crate::environment::Environment;
use crate::error::model::Error;
use crate::state::app::{AppFocus, Tool};
use crate::state::token_generator::Focus;
use ratatui::crossterm::event::Event as CrosstermEvent;

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

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    // List events
    ListMove(Direction), // Move up and down tool List
    ListSelect(Tool),    // Select item on tool list
    SystemError(Error),
    DismissError,

    // Tool events
    // ServiceStatus
    ServiceStatusListMove(Direction),
    GetCommitRefOk(String, usize, Environment),
    GetCommitRefErrored(String, usize, Environment),
    ScanServices, // Scan all services
    ScanServiceEnv(usize, Environment),

    // TokenGenerator
    TokenGenServiceListMove(Direction),
    TokenGenEnvListMove(Direction),
    SetTokenGenFocus(Focus),
    GenerateToken,
    TokenGenerated(String, usize, usize),
    TokenFailed(String, usize, usize),

    // Jira
    JiraTicketListMove(Direction), // Move down ticket list
    NewJiraTicket,
    AddTicketIdChar(char),
    RemoveTicketIdChar,
    SubmitTicketId,
    RemoveTicket,
    JiraTicketMove(Direction), // Move selected ticket up and down list
    TicketRetrieved(TicketResponse),
    JiraTicketListUpdate,
    ScanTickets,

    // Log events
    LogsListMove(Direction),
    ActivityEvent(String, String), // source, message
    AppLog(crate::state::log::LogLevel, String, String), // level, source, message
    OpenLogs,

    // Generic Events
    SetFocus(AppFocus),
    Quit,
    OpenInBrowser,
    CopyToClipboard,

    // Config events
    ConfigListMove(Direction),
    ToggleFeature,
    OpenToolConfig(Tool),
    CloseToolConfig,

    // Service Status config events
    ServiceStatusConfigListMove(Direction),
    OpenAddService,
    OpenEditService,
    ServiceStatusFormNextField,
    ServiceStatusFormPrevField,
    ServiceStatusFormChar(char),
    ServiceStatusFormBackspace,
    SubmitServiceConfig,
    RemoveService,

    // Token Generator config events
    TokenGenConfigListMove(Direction),
    OpenAddTokenGenService,
    TokenGenConfigFormNextField,
    TokenGenConfigFormPrevField,
    TokenGenConfigFormChar(char),
    TokenGenConfigFormBackspace,
    SubmitTokenGenConfig,
    RemoveTokenGenService,
    TgConfigSwitchFocus,
    TgConfigEdit,
    // Jira config events
    OpenJiraConfigEdit,
    JiraConfigFormNextField,
    JiraConfigFormPrevField,
    JiraConfigFormChar(char),
    JiraConfigFormBackspace,
    SubmitJiraConfig,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Direction {
    Up,
    Down,
}
