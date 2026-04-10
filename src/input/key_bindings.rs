use crate::app::AppFocus;
use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
use crate::events::event::AppEvent::{
    AddTicketIdChar, CloseToolConfig, ConfigListMove, CopyToClipboard, DismissPopup,
    GenerateToken, JiraConfigPopupBackspace, JiraConfigPopupChar, JiraConfigPopupNextField,
    JiraConfigPopupPrevField, JiraTicketListMove, JiraTicketMove, ListMove, NewJiraTicketPopUp,
    OpenAddServicePopup, OpenAddTokenGenServicePopup, OpenEditServicePopup,
    OpenInBrowser, OpenJiraConfigPopup, OpenToolConfig, Quit,
    RemoveService, RemoveTicket, RemoveTicketIdChar, RemoveTokenGenService, ScanServices,
    ServiceStatusConfigListMove, ServiceStatusListMove, ServiceStatusPopupBackspace,
    ServiceStatusPopupChar, ServiceStatusPopupNextField, ServiceStatusPopupPrevField, SetFocus,
    SetTokenGenFocus, SubmitJiraConfig, SubmitServiceConfig, SubmitTicketId, SubmitTokenGenConfig,
    TgConfigEdit, TgConfigSwitchFocus, ToggleFeature, TokenGenConfigListMove,
    TokenGenConfigPopupBackspace, TokenGenConfigPopupChar, TokenGenConfigPopupNextField,
    TokenGenConfigPopupPrevField, TokenGenEnvListMove, TokenGenServiceListMove,
};
use crate::events::event::{AppEvent, Direction};
use crate::input::key_context::KeyContext::{
    Config, ErrorPopUp, Global, List, Popup, TokenGen, Tool, ToolConfig, ToolConfigPopup,
    ToolIgnore,
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
        KeyCode::Char('1'),
        KeyModifiers::NONE,
        SetFocus(AppFocus::List),
    );
    key_event_map.add_static(
        Global,
        KeyCode::Char('2'),
        KeyModifiers::NONE,
        SetFocus(AppFocus::Config),
    );
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

    // CONFIG EVENTS
    key_event_map.add_static(
        Config,
        KeyCode::Down,
        KeyModifiers::NONE,
        ConfigListMove(Direction::Down),
    );
    key_event_map.add_static(
        Config,
        KeyCode::Up,
        KeyModifiers::NONE,
        ConfigListMove(Direction::Up),
    );
    key_event_map.add_static(Config, KeyCode::Enter, KeyModifiers::NONE, ToggleFeature);
    key_event_map.add_static(
        Config,
        KeyCode::Right,
        KeyModifiers::NONE,
        OpenToolConfig(ServiceStatus),
    );
    key_event_map.add_static(
        Config,
        KeyCode::Left,
        KeyModifiers::NONE,
        SetFocus(AppFocus::List),
    );

    // TOOL CONFIG EVENTS (Service Status)
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Down,
        KeyModifiers::NONE,
        ServiceStatusConfigListMove(Direction::Down),
    );
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Up,
        KeyModifiers::NONE,
        ServiceStatusConfigListMove(Direction::Up),
    );
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        OpenAddServicePopup,
    );
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        OpenEditServicePopup,
    );
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        RemoveService,
    );
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Left,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );

    // SERVICE STATUS ADD POPUP EVENTS
    key_event_map.add_static(
        Popup(ServiceStatus),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        Popup(ServiceStatus),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitServiceConfig,
    );
    key_event_map.add_static(
        Popup(ServiceStatus),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        ServiceStatusPopupBackspace,
    );
    key_event_map.add_static(
        Popup(ServiceStatus),
        KeyCode::Down,
        KeyModifiers::NONE,
        ServiceStatusPopupNextField,
    );
    key_event_map.add_static(
        Popup(ServiceStatus),
        KeyCode::Up,
        KeyModifiers::NONE,
        ServiceStatusPopupPrevField,
    );
    key_event_map.add_static(
        Popup(ServiceStatus),
        KeyCode::Tab,
        KeyModifiers::NONE,
        ServiceStatusPopupNextField,
    );
    key_event_map.add_static(
        Popup(ServiceStatus),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        ServiceStatusPopupPrevField,
    );
    key_event_map.add_dynamic(Popup(ServiceStatus), service_status_popup_char);

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
        KeyCode::Right,
        KeyModifiers::NONE,
        SetFocus(AppFocus::Tool),
    );
    key_event_map.add_static(
        List,
        KeyCode::Down,
        KeyModifiers::NONE,
        ListMove(Direction::Down),
    );
    key_event_map.add_static(
        List,
        KeyCode::Up,
        KeyModifiers::NONE,
        ListMove(Direction::Up),
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

    // TOKEN GENERATOR CONFIG EVENTS
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGenConfigListMove(Direction::Down),
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGenConfigListMove(Direction::Up),
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        OpenAddTokenGenServicePopup,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        TgConfigEdit,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        RemoveTokenGenService,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Tab,
        KeyModifiers::NONE,
        TgConfigSwitchFocus,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        TgConfigSwitchFocus,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Left,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );

    // TOKEN GENERATOR CONFIG POPUP EVENTS
    key_event_map.add_static(
        Popup(TokenGenerator),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        Popup(TokenGenerator),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitTokenGenConfig,
    );
    key_event_map.add_static(
        Popup(TokenGenerator),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        TokenGenConfigPopupBackspace,
    );
    key_event_map.add_static(
        Popup(TokenGenerator),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGenConfigPopupNextField,
    );
    key_event_map.add_static(
        Popup(TokenGenerator),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGenConfigPopupPrevField,
    );
    key_event_map.add_static(
        Popup(TokenGenerator),
        KeyCode::Tab,
        KeyModifiers::NONE,
        TokenGenConfigPopupNextField,
    );
    key_event_map.add_static(
        Popup(TokenGenerator),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        TokenGenConfigPopupPrevField,
    );
    key_event_map.add_dynamic(Popup(TokenGenerator), token_gen_config_popup_char);

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

    // JIRA CONFIG EVENTS
    key_event_map.add_static(
        ToolConfig(Jira),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        OpenJiraConfigPopup,
    );
    key_event_map.add_static(
        ToolConfig(Jira),
        KeyCode::Left,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        ToolConfig(Jira),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );

    // JIRA CONFIG POPUP EVENTS
    key_event_map.add_static(
        ToolConfigPopup(Jira),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        ToolConfigPopup(Jira),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitJiraConfig,
    );
    key_event_map.add_static(
        ToolConfigPopup(Jira),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        JiraConfigPopupBackspace,
    );
    key_event_map.add_static(
        ToolConfigPopup(Jira),
        KeyCode::Down,
        KeyModifiers::NONE,
        JiraConfigPopupNextField,
    );
    key_event_map.add_static(
        ToolConfigPopup(Jira),
        KeyCode::Up,
        KeyModifiers::NONE,
        JiraConfigPopupPrevField,
    );
    key_event_map.add_static(
        ToolConfigPopup(Jira),
        KeyCode::Tab,
        KeyModifiers::NONE,
        JiraConfigPopupNextField,
    );
    key_event_map.add_static(
        ToolConfigPopup(Jira),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        JiraConfigPopupPrevField,
    );
    key_event_map.add_dynamic(ToolConfigPopup(Jira), jira_config_popup_char);
}

fn add_ticket_id_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(AddTicketIdChar)
}

fn service_status_popup_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(ServiceStatusPopupChar)
}

fn token_gen_config_popup_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(TokenGenConfigPopupChar)
}

fn jira_config_popup_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(JiraConfigPopupChar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
    use crate::events::event::Direction::{Down, Up};
    use crate::input::key_context::KeyContext;
    use crate::input::key_context::KeyContext::{Config, Global, List, Popup, TokenGen, Tool, ToolConfig, ToolIgnore};
    use crate::state::token_generator::Focus;
    use test_case::test_case;

    fn registered_map() -> KeyEventMap {
        let mut map = KeyEventMap::default();
        register_bindings(&mut map);
        map
    }

    #[test_case(Global, KeyCode::Char('q'), KeyModifiers::NONE, Quit; "q quits")]
    #[test_case(Global, KeyCode::Esc, KeyModifiers::NONE, Quit; "esc quits")]
    #[test_case(Global, KeyCode::Char('1'), KeyModifiers::NONE, SetFocus(AppFocus::List); "1 focuses tools list")]
    #[test_case(Global, KeyCode::Char('2'), KeyModifiers::NONE, SetFocus(AppFocus::Config); "2 focuses config")]
    #[test_case(Global, KeyCode::Char('c'), KeyModifiers::NONE, CopyToClipboard; "c copies")]
    #[test_case(Global, KeyCode::Char('o'), KeyModifiers::NONE, OpenInBrowser; "o opens browser")]
    #[test_case(ErrorPopUp, KeyCode::Char('d'), KeyModifiers::NONE, DismissPopup; "popup dismissed")]
    #[test_case(Config, KeyCode::Down, KeyModifiers::NONE, AppEvent::ConfigListMove(Down); "config down")]
    #[test_case(Config, KeyCode::Up, KeyModifiers::NONE, AppEvent::ConfigListMove(Up); "config up")]
    #[test_case(Config, KeyCode::Enter, KeyModifiers::NONE, ToggleFeature; "config enter toggles feature")]
    #[test_case(Config, KeyCode::Left, KeyModifiers::NONE, SetFocus(AppFocus::List); "config left focuses tools list")]
    #[test_case(Config, KeyCode::Right, KeyModifiers::NONE, OpenToolConfig(ServiceStatus); "config right opens tool config")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Down, KeyModifiers::NONE, AppEvent::ServiceStatusConfigListMove(Down); "tool config down")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Up, KeyModifiers::NONE, AppEvent::ServiceStatusConfigListMove(Up); "tool config up")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Char('a'), KeyModifiers::NONE, OpenAddServicePopup; "tool config a opens add popup")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Char('x'), KeyModifiers::NONE, RemoveService; "tool config x removes service")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Left, KeyModifiers::NONE, CloseToolConfig; "tool config left closes")]
    #[test_case(Popup(ServiceStatus), KeyCode::Enter, KeyModifiers::NONE, SubmitServiceConfig; "service popup enter submits")]
    #[test_case(Popup(ServiceStatus), KeyCode::Backspace, KeyModifiers::NONE, ServiceStatusPopupBackspace; "service popup backspace")]
    #[test_case(Popup(ServiceStatus), KeyCode::Tab, KeyModifiers::NONE, ServiceStatusPopupNextField; "service popup tab next field")]
    #[test_case(Popup(ServiceStatus), KeyCode::BackTab, KeyModifiers::SHIFT, ServiceStatusPopupPrevField; "service popup shift-tab prev field")]
    #[test_case(List, KeyCode::Right, KeyModifiers::NONE, SetFocus(AppFocus::Tool); "list right focuses tool")]
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
        let map = registered_map();
        let result = map.resolve(context, KeyEvent::new(code, modifiers));
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn popup_dynamic_handler_maps_char_to_add_ticket_id_char() {
        let map = registered_map();
        let result = map.resolve(
            Popup(Jira),
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE),
        );
        assert_eq!(result, Some(AddTicketIdChar('A')));
    }

    #[test]
    fn popup_dynamic_handler_returns_none_for_non_char() {
        let map = registered_map();
        let result = map.resolve(Popup(Jira), KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(result, None);
    }
}
