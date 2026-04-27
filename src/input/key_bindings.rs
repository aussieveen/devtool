use crate::app::AppFocus;
use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
use crate::event::event::AppEvent::{
    AddTicketIdChar, CloseToolConfig, ConfigListMove, CopyToClipboard, DismissError, GenerateToken,
    JiraConfigFormBackspace, JiraConfigFormChar, JiraConfigFormNextField, JiraConfigFormPrevField,
    JiraTicketListMove, JiraTicketMove, ListMove, LogsListMove, NewJiraTicket, OpenAddService,
    OpenAddTokenGenService, OpenEditService, OpenInBrowser, OpenJiraConfigEdit, OpenLogs,
    OpenToolConfig, Quit, RemoveService, RemoveTicket, RemoveTicketIdChar, RemoveTokenGenService,
    ScanServices, ServiceStatusConfigListMove, ServiceStatusFormBackspace, ServiceStatusFormChar,
    ServiceStatusFormNextField, ServiceStatusFormPrevField, ServiceStatusListMove, SetFocus,
    SetTokenGenFocus, SubmitJiraConfig, SubmitServiceConfig, SubmitTicketId, SubmitTokenGenConfig,
    TokenGeneratorConfigEdit, TokenGeneratorConfigSwitchFocus, ToggleFeature, TokenGenConfigFormBackspace,
    TokenGenConfigFormChar, TokenGenConfigFormNextField, TokenGenConfigFormPrevField,
    TokenGenConfigListMove, TokenGenEnvListMove, TokenGenServiceListMove,
};
use crate::event::event::{AppEvent, Direction};
use crate::input::key_context::KeyContext::{
    Config, Editing, Error, Global, List, Logs, TokenGen, Tool, ToolConfig, ToolConfigEditing,
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
    key_event_map.add_static(Global, KeyCode::Char('3'), KeyModifiers::NONE, OpenLogs);
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
    key_event_map.add_static(Error, KeyCode::Char('d'), KeyModifiers::NONE, DismissError);

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

    // LOGS EVENTS
    key_event_map.add_static(
        Logs,
        KeyCode::Down,
        KeyModifiers::NONE,
        LogsListMove(Direction::Down),
    );
    key_event_map.add_static(
        Logs,
        KeyCode::Up,
        KeyModifiers::NONE,
        LogsListMove(Direction::Up),
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
        OpenAddService,
    );
    key_event_map.add_static(
        ToolConfig(ServiceStatus),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        OpenEditService,
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
        Editing(ServiceStatus),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        Editing(ServiceStatus),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitServiceConfig,
    );
    key_event_map.add_static(
        Editing(ServiceStatus),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        ServiceStatusFormBackspace,
    );
    key_event_map.add_static(
        Editing(ServiceStatus),
        KeyCode::Down,
        KeyModifiers::NONE,
        ServiceStatusFormNextField,
    );
    key_event_map.add_static(
        Editing(ServiceStatus),
        KeyCode::Up,
        KeyModifiers::NONE,
        ServiceStatusFormPrevField,
    );
    key_event_map.add_static(
        Editing(ServiceStatus),
        KeyCode::Tab,
        KeyModifiers::NONE,
        ServiceStatusFormNextField,
    );
    key_event_map.add_static(
        Editing(ServiceStatus),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        ServiceStatusFormPrevField,
    );
    key_event_map.add_dynamic(Editing(ServiceStatus), service_status_form_char);

    // POP UP EVENTS
    key_event_map.add_static(
        Editing(Jira),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        RemoveTicketIdChar,
    );
    key_event_map.add_static(
        Editing(Jira),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitTicketId,
    );
    key_event_map.add_dynamic(Editing(Jira), add_ticket_id_char);

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
        OpenAddTokenGenService,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        TokenGeneratorConfigEdit,
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
        TokenGeneratorConfigSwitchFocus,
    );
    key_event_map.add_static(
        ToolConfig(TokenGenerator),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        TokenGeneratorConfigSwitchFocus,
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
        Editing(TokenGenerator),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        Editing(TokenGenerator),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitTokenGenConfig,
    );
    key_event_map.add_static(
        Editing(TokenGenerator),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        TokenGenConfigFormBackspace,
    );
    key_event_map.add_static(
        Editing(TokenGenerator),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGenConfigFormNextField,
    );
    key_event_map.add_static(
        Editing(TokenGenerator),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGenConfigFormPrevField,
    );
    key_event_map.add_static(
        Editing(TokenGenerator),
        KeyCode::Tab,
        KeyModifiers::NONE,
        TokenGenConfigFormNextField,
    );
    key_event_map.add_static(
        Editing(TokenGenerator),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        TokenGenConfigFormPrevField,
    );
    key_event_map.add_dynamic(Editing(TokenGenerator), token_gen_config_form_char);

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
        NewJiraTicket,
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
        OpenJiraConfigEdit,
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
        ToolConfigEditing(Jira),
        KeyCode::Esc,
        KeyModifiers::NONE,
        CloseToolConfig,
    );
    key_event_map.add_static(
        ToolConfigEditing(Jira),
        KeyCode::Enter,
        KeyModifiers::NONE,
        SubmitJiraConfig,
    );
    key_event_map.add_static(
        ToolConfigEditing(Jira),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        JiraConfigFormBackspace,
    );
    key_event_map.add_static(
        ToolConfigEditing(Jira),
        KeyCode::Down,
        KeyModifiers::NONE,
        JiraConfigFormNextField,
    );
    key_event_map.add_static(
        ToolConfigEditing(Jira),
        KeyCode::Up,
        KeyModifiers::NONE,
        JiraConfigFormPrevField,
    );
    key_event_map.add_static(
        ToolConfigEditing(Jira),
        KeyCode::Tab,
        KeyModifiers::NONE,
        JiraConfigFormNextField,
    );
    key_event_map.add_static(
        ToolConfigEditing(Jira),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        JiraConfigFormPrevField,
    );
    key_event_map.add_dynamic(ToolConfigEditing(Jira), jira_config_form_char);
}

fn add_ticket_id_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(AddTicketIdChar)
}

fn service_status_form_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(ServiceStatusFormChar)
}

fn token_gen_config_form_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(TokenGenConfigFormChar)
}

fn jira_config_form_char(key_event: KeyEvent) -> Option<AppEvent> {
    key_event.code.as_char().map(JiraConfigFormChar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Tool::{Jira, ServiceStatus, TokenGenerator};
    use crate::event::event::Direction::{Down, Up};
    use crate::input::key_context::KeyContext;
    use crate::input::key_context::KeyContext::{
        Config, Editing, Global, List, Logs, TokenGen, Tool, ToolConfig, ToolIgnore,
    };
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
    #[test_case(Global, KeyCode::Char('3'), KeyModifiers::NONE, OpenLogs; "3 opens logs")]
    #[test_case(Global, KeyCode::Char('c'), KeyModifiers::NONE, CopyToClipboard; "c copies")]
    #[test_case(Global, KeyCode::Char('o'), KeyModifiers::NONE, OpenInBrowser; "o opens browser")]
    #[test_case(Error, KeyCode::Char('d'), KeyModifiers::NONE, DismissError; "error dismissed")]
    #[test_case(Config, KeyCode::Down, KeyModifiers::NONE, AppEvent::ConfigListMove(Down); "config down")]
    #[test_case(Config, KeyCode::Up, KeyModifiers::NONE, AppEvent::ConfigListMove(Up); "config up")]
    #[test_case(Config, KeyCode::Enter, KeyModifiers::NONE, ToggleFeature; "config enter toggles feature")]
    #[test_case(Config, KeyCode::Left, KeyModifiers::NONE, SetFocus(AppFocus::List); "config left focuses tools list")]
    #[test_case(Config, KeyCode::Right, KeyModifiers::NONE, OpenToolConfig(ServiceStatus); "config right opens tool config")]
    #[test_case(Logs, KeyCode::Down, KeyModifiers::NONE, AppEvent::LogsListMove(Down); "logs down navigates")]
    #[test_case(Logs, KeyCode::Up, KeyModifiers::NONE, AppEvent::LogsListMove(Up); "logs up navigates")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Down, KeyModifiers::NONE, AppEvent::ServiceStatusConfigListMove(Down); "tool config down")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Up, KeyModifiers::NONE, AppEvent::ServiceStatusConfigListMove(Up); "tool config up")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Char('a'), KeyModifiers::NONE, OpenAddService; "tool config a opens add form")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Char('x'), KeyModifiers::NONE, RemoveService; "tool config x removes service")]
    #[test_case(ToolConfig(ServiceStatus), KeyCode::Left, KeyModifiers::NONE, CloseToolConfig; "tool config left closes")]
    #[test_case(Editing(ServiceStatus), KeyCode::Enter, KeyModifiers::NONE, SubmitServiceConfig; "service form enter submits")]
    #[test_case(Editing(ServiceStatus), KeyCode::Backspace, KeyModifiers::NONE, ServiceStatusFormBackspace; "service form backspace")]
    #[test_case(Editing(ServiceStatus), KeyCode::Tab, KeyModifiers::NONE, ServiceStatusFormNextField; "service form tab next field")]
    #[test_case(Editing(ServiceStatus), KeyCode::BackTab, KeyModifiers::SHIFT, ServiceStatusFormPrevField; "service form shift-tab prev field")]
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
    #[test_case(Tool(Jira), KeyCode::Char('a'), KeyModifiers::NONE, NewJiraTicket; "jira a adds ticket")]
    #[test_case(Tool(Jira), KeyCode::Char('x'), KeyModifiers::NONE, RemoveTicket; "jira x removes ticket")]
    #[test_case(Editing(Jira), KeyCode::Backspace, KeyModifiers::NONE, RemoveTicketIdChar; "form backspace removes char")]
    #[test_case(Editing(Jira), KeyCode::Enter, KeyModifiers::NONE, SubmitTicketId; "form enter submits")]
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
            Editing(Jira),
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE),
        );
        assert_eq!(result, Some(AddTicketIdChar('A')));
    }

    #[test]
    fn popup_dynamic_handler_returns_none_for_non_char() {
        let map = registered_map();
        let result = map.resolve(
            Editing(Jira),
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        );
        assert_eq!(result, None);
    }
}
