use crate::app::AppFocus;
use crate::app::Tool::{Home, Jira, ServiceStatus, TokenGenerator};
use crate::events::event::AppEvent::{
    AddTicketIdChar, CopyToClipboard, GenerateToken, JiraTicketListMove, JiraTicketMove,
    NewJiraTicketPopUp, OpenInBrowser, Quit, RemoveTicket, RemoveTicketIdChar, ScanServices,
    ServiceStatusListMove, SetFocus, SetTokenGenFocus, SubmitTicketId, TokenGenEnvListMove,
    TokenGenServiceListMove,
};
use crate::events::event::{AppEvent, Direction};
use crate::input::key_context::KeyContext::{
    Global, List, ListIgnore, Popup, TokenGen, Tool, ToolIgnore,
};
use crate::input::key_event_map::KeyEventMap;
use crate::state::token_generator::Focus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn register_bindings(key_event_map: &mut KeyEventMap) {
    // GLOBAL EVENTS
    key_event_map.add_static(Global, KeyCode::Char('q'), KeyModifiers::NONE, Quit);
    key_event_map.add_static(Global, KeyCode::Esc, KeyModifiers::NONE, Quit);
    key_event_map.add_static(
        Global,
        KeyCode::Char('c'),
        KeyModifiers::NONE,
        CopyToClipboard,
    );
    key_event_map.add_static(
        Global,
        KeyCode::Char('o'),
        KeyModifiers::NONE,
        OpenInBrowser,
    );

    // POP UP EVENTS
    key_event_map.add_static(
        Popup(Jira),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        RemoveTicketIdChar,
    );
    key_event_map.add_static(
        Popup(Jira),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitTicketId,
    );
    key_event_map.add_dynamic(Popup(Jira), add_ticket_id_char);

    // LIST EVENTS
    key_event_map.add_static(
        ListIgnore(Home),
        KeyCode::Right,
        KeyModifiers::NONE,
        AppEvent::SetFocus(AppFocus::Tool),
    );
    key_event_map.add_static(
        List,
        KeyCode::Down,
        KeyModifiers::NONE,
        AppEvent::ListMove(Direction::Down),
    );
    key_event_map.add_static(
        List,
        KeyCode::Up,
        KeyModifiers::NONE,
        AppEvent::ListMove(Direction::Up),
    );

    // SERVICE STATUS EVENTS
    key_event_map.add_static(
        Tool(ServiceStatus),
        KeyCode::Down,
        KeyModifiers::NONE,
        ServiceStatusListMove(Direction::Down),
    );
    key_event_map.add_static(
        Tool(ServiceStatus),
        KeyCode::Up,
        KeyModifiers::NONE,
        ServiceStatusListMove(Direction::Up),
    );
    key_event_map.add_static(
        Tool(ServiceStatus),
        KeyCode::Char('s'),
        KeyModifiers::NONE,
        ScanServices,
    );

    // TOKEN GENERATOR EVENTS
    key_event_map.add_static(
        ToolIgnore(TokenGenerator),
        KeyCode::Left,
        KeyModifiers::NONE,
        SetFocus(AppFocus::List),
    );
    key_event_map.add_static(
        TokenGen(Focus::Service),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGenServiceListMove(Direction::Down),
    );
    key_event_map.add_static(
        TokenGen(Focus::Service),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGenServiceListMove(Direction::Up),
    );
    key_event_map.add_static(
        TokenGen(Focus::Env),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGenEnvListMove(Direction::Down),
    );
    key_event_map.add_static(
        TokenGen(Focus::Env),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGenEnvListMove(Direction::Up),
    );
    key_event_map.add_static(
        Tool(TokenGenerator),
        KeyCode::Right,
        KeyModifiers::NONE,
        SetTokenGenFocus(Focus::Env),
    );
    key_event_map.add_static(
        TokenGen(Focus::Service),
        KeyCode::Left,
        KeyModifiers::NONE,
        SetFocus(AppFocus::List),
    );
    key_event_map.add_static(
        TokenGen(Focus::Env),
        KeyCode::Left,
        KeyModifiers::NONE,
        SetTokenGenFocus(Focus::Service),
    );
    key_event_map.add_static(
        Tool(TokenGenerator),
        KeyCode::Enter,
        KeyModifiers::NONE,
        GenerateToken,
    );
    key_event_map.add_static(
        Tool(TokenGenerator),
        KeyCode::Char('c'),
        KeyModifiers::NONE,
        CopyToClipboard,
    );

    // JIRA EVENTS
    key_event_map.add_static(
        Tool(Jira),
        KeyCode::Up,
        KeyModifiers::NONE,
        JiraTicketListMove(Direction::Up),
    );
    key_event_map.add_static(
        Tool(Jira),
        KeyCode::Down,
        KeyModifiers::NONE,
        JiraTicketListMove(Direction::Down),
    );
    key_event_map.add_static(
        Tool(Jira),
        KeyCode::Up,
        KeyModifiers::SHIFT,
        JiraTicketMove(Direction::Up),
    );
    key_event_map.add_static(
        Tool(Jira),
        KeyCode::Down,
        KeyModifiers::SHIFT,
        JiraTicketMove(Direction::Down),
    );
    key_event_map.add_static(
        Tool(Jira),
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        NewJiraTicketPopUp,
    );
    key_event_map.add_static(
        Tool(Jira),
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        RemoveTicket,
    );
}

fn add_ticket_id_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(AddTicketIdChar)
}
