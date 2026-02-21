use crate::app::AppFocus;
use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
use crate::events::event::AppEvent::{
    AddTicketIdChar, CopyToClipboard, DismissPopup, GenerateToken, JiraTicketListMove,
    JiraTicketMove, NewJiraTicketPopUp, OpenInBrowser, Quit, RemoveTicket, RemoveTicketIdChar,
    ScanServices, ServiceStatusListMove, SetFocus, SetTokenGenFocus, SubmitTicketId,
    TokenGenEnvListMove, TokenGenServiceListMove,
};
use crate::events::event::{AppEvent, Direction};
use crate::input::key_context::KeyContext::{
    ErrorPopUp, Global, List, ListIgnore, Popup, TokenGen, Tool, ToolIgnore,
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
    key_event_map.add_static(
        ErrorPopUp,
        KeyCode::Char('d'),
        KeyModifiers::NONE,
        DismissPopup,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
    use crate::events::event::Direction::{Down, Up};
    use crate::input::key_context::KeyContext;
    use crate::input::key_context::KeyContext::{
        Global, List, ListIgnore, Popup, TokenGen, Tool, ToolIgnore,
    };
    use crate::state::token_generator::Focus;
    use test_case::test_case;

    fn registered_map() -> KeyEventMap {
        let mut map = KeyEventMap::new();
        register_bindings(&mut map);
        map
    }

    #[test_case(Global, KeyCode::Char('q'), KeyModifiers::NONE, Quit; "q quits")]
    #[test_case(Global, KeyCode::Esc, KeyModifiers::NONE, Quit; "esc quits")]
    #[test_case(Global, KeyCode::Char('c'), KeyModifiers::NONE, CopyToClipboard; "c copies")]
    #[test_case(Global, KeyCode::Char('o'), KeyModifiers::NONE, OpenInBrowser; "o opens browser")]
    #[test_case(ErrorPopUp, KeyCode::Char('d'), KeyModifiers::NONE, DismissPopup; "popup dismissed")]
    #[test_case(List, KeyCode::Down, KeyModifiers::NONE, AppEvent::ListMove(Down); "list down")]
    #[test_case(List, KeyCode::Up, KeyModifiers::NONE, AppEvent::ListMove(Up); "list up")]
    #[test_case(Tool(ServiceStatus), KeyCode::Down, KeyModifiers::NONE, ServiceStatusListMove(Down); "service status down")]
    #[test_case(Tool(ServiceStatus), KeyCode::Up, KeyModifiers::NONE, ServiceStatusListMove(Up); "service status up")]
    #[test_case(Tool(ServiceStatus), KeyCode::Char('s'), KeyModifiers::NONE, ScanServices; "s scans services")]
    #[test_case(ToolIgnore(TokenGenerator), KeyCode::Left, KeyModifiers::NONE, SetFocus(AppFocus::List); "tool left focuses list")]
    #[test_case(TokenGen(Focus::Service), KeyCode::Down, KeyModifiers::NONE, TokenGenServiceListMove(Down); "token service down")]
    #[test_case(TokenGen(Focus::Service), KeyCode::Up, KeyModifiers::NONE, TokenGenServiceListMove(Up); "token service up")]
    #[test_case(TokenGen(Focus::Env), KeyCode::Down, KeyModifiers::NONE, TokenGenEnvListMove(Down); "token env down")]
    #[test_case(TokenGen(Focus::Env), KeyCode::Up, KeyModifiers::NONE, TokenGenEnvListMove(Up); "token env up")]
    #[test_case(Tool(TokenGenerator), KeyCode::Right, KeyModifiers::NONE, SetTokenGenFocus(Focus::Env); "token right focuses env")]
    #[test_case(TokenGen(Focus::Service), KeyCode::Left, KeyModifiers::NONE, SetFocus(AppFocus::List); "token service left focuses list")]
    #[test_case(TokenGen(Focus::Env), KeyCode::Left, KeyModifiers::NONE, SetTokenGenFocus(Focus::Service); "token env left focuses service")]
    #[test_case(Tool(TokenGenerator), KeyCode::Enter, KeyModifiers::NONE, GenerateToken; "token enter generates")]
    #[test_case(Tool(Jira), KeyCode::Up, KeyModifiers::NONE, JiraTicketListMove(Up); "jira up")]
    #[test_case(Tool(Jira), KeyCode::Down, KeyModifiers::NONE, JiraTicketListMove(Down); "jira down")]
    #[test_case(Tool(Jira), KeyCode::Up, KeyModifiers::SHIFT, JiraTicketMove(Up); "jira shift up moves ticket")]
    #[test_case(Tool(Jira), KeyCode::Down, KeyModifiers::SHIFT, JiraTicketMove(Down); "jira shift down moves ticket")]
    #[test_case(Tool(Jira), KeyCode::Char('a'), KeyModifiers::NONE, NewJiraTicketPopUp; "jira a adds ticket")]
    #[test_case(Tool(Jira), KeyCode::Char('x'), KeyModifiers::NONE, RemoveTicket; "jira x removes ticket")]
    #[test_case(Popup(Jira), KeyCode::Backspace, KeyModifiers::NONE, RemoveTicketIdChar; "popup backspace removes char")]
    #[test_case(Popup(Jira), KeyCode::Enter, KeyModifiers::NONE, SubmitTicketId; "popup enter submits")]
    fn binding_resolves_to_expected_event(
        context: KeyContext,
        code: KeyCode,
        modifiers: KeyModifiers,
        expected: AppEvent,
    ) {
        let mut map = registered_map();
        let result = map.resolve(context, KeyEvent::new(code, modifiers));
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn popup_dynamic_handler_maps_char_to_add_ticket_id_char() {
        let mut map = registered_map();
        let result = map.resolve(
            Popup(Jira),
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE),
        );
        assert_eq!(result, Some(AddTicketIdChar('A')));
    }

    #[test]
    fn popup_dynamic_handler_returns_none_for_non_char() {
        let mut map = registered_map();
        let result = map.resolve(Popup(Jira), KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(result, None);
    }
}
