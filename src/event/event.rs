use crate::client::jira::models::TicketResponse;
use crate::environment::Environment;
use crate::error::model::Error;
use crate::state::app::{AppFocus, Tool};
use crate::state::token_generator::Focus;
use ratatui::crossterm::event::Event as CrosstermEvent;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Crossterm event.
    ///
    /// These event are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application event.
    ///
    /// Use this event to emit custom event that are specific to your application.
    App(AppEvent),
    Generic(GenericEvent),
    ServiceStatus(ServiceStatusEvent),
    ServiceStatusConfig(ServiceStatusConfigEvent),
    TokenGenerator(TokenGeneratorEvent),
    TokenGeneratorConfig(TokenGeneratorConfigEvent),
    Jira(JiraEvent),
    JiraConfig(JiraConfigEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    // List event
    ListMove(Direction), // Move up and down tool List
    ListSelect(Tool),    // Select item on tool list
    SystemError(Error),
    DismissError,

    // Log event
    LogsListMove(Direction),
    ActivityEvent(String, String), // source, message
    AppLog(crate::state::log::LogLevel, String, String), // level, source, message
    OpenLogs,

    // Config event
    ConfigListMove(Direction),
    ToggleFeature,
    OpenToolConfig(Tool),
    CloseToolConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GenericEvent {
    SetFocus(AppFocus),
    Quit,
    OpenInBrowser,
    CopyToClipboard,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServiceStatusEvent {
    ListMove(Direction),
    GetCommitRefOk(String, usize, Environment),
    GetCommitRefErrored(String, usize, Environment),
    Scan, // Scan all services
    ScanServiceEnv(usize, Environment),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServiceStatusConfigEvent {
    ListMove(Direction),
    OpenAddService,
    OpenEditService,
    FormNextField,
    PrevField,
    FormChar(char),
    FormBackspace,
    SubmitConfig,
    RemoveService
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenGeneratorEvent {
    ServiceListMove(Direction),
    EnvListMove(Direction),
    SetFocus(Focus),
    GenerateToken,
    TokenGenerated(String, usize, usize),
    TokenFailed(String, usize, usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenGeneratorConfigEvent {
    ConfigListMove(Direction),
    OpenAddService,
    FormNextField,
    FormPrevField,
    FormChar(char),
    FormBackspace,
    SubmitConfig,
    RemoveService,
    SwitchFocus,
    ConfigEdit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JiraEvent {
    ListMove(Direction), // Move down ticket list
    NewTicket,
    AddTicketIdChar(char),
    RemoveTicketIdChar,
    SubmitTicketId,
    RemoveTicket,
    TicketMove(Direction), // Move selected ticket up and down list
    TicketRetrieved(TicketResponse),
    TicketListUpdate,
    ScanTickets,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JiraConfigEvent {
    OpenEdit,
    FormNextField,
    FormPrevField,
    FormChar(char),
    FormBackspace,
    SubmitConfig,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Direction {
    Up,
    Down,
}

impl From<AppEvent> for Event {
    fn from(e: AppEvent) -> Self { Event::App(e) }
}

impl From<GenericEvent> for Event {
    fn from(e: GenericEvent) -> Self { Event::Generic(e) }
}

impl From<ServiceStatusEvent> for Event {
    fn from(e: ServiceStatusEvent) -> Self { Event::ServiceStatus(e) }
}

impl From<ServiceStatusConfigEvent> for Event {
    fn from(e: ServiceStatusConfigEvent) -> Self { Event::ServiceStatusConfig(e) }
}

impl From<TokenGeneratorEvent> for Event {
    fn from(e: TokenGeneratorEvent) -> Self { Event::TokenGenerator(e) }
}

impl From<TokenGeneratorConfigEvent> for Event {
    fn from(e: TokenGeneratorConfigEvent) -> Self { Event::TokenGeneratorConfig(e) }
}

impl From<JiraEvent> for Event {
    fn from(e: JiraEvent) -> Self { Event::Jira(e) }
}

impl From<JiraConfigEvent> for Event {
    fn from(e: JiraConfigEvent) -> Self { Event::JiraConfig(e) }
}
