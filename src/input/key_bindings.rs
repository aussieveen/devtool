use crate::app::{AppFocus, Tool};
use crate::event::events::{
    AppEvent as App, Direction, Event, GenericEvent as Generic, JiraConfigEvent as JiraConfig,
    JiraEvent as Jira, ServiceStatusConfigEvent as ServiceStatusConfig,
    ServiceStatusEvent as ServiceStatus, TokenGeneratorConfigEvent as TokenGenConfig,
    TokenGeneratorEvent as TokenGen,
};
use crate::input::key_context::KeyContext::{
    Config, Editing, Global, List, Logs, Popup, TokenGen as TokenGenCtx, Tool as ToolCtx,
    ToolConfig, ToolConfigEditing, ToolIgnore,
};
use crate::input::key_event_map::KeyEventMap;
use crate::state::token_generator::Focus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn register_bindings(key_event_map: &mut KeyEventMap) {
    // GLOBAL EVENTS
    key_event_map.add_static(
        Global,
        KeyCode::Char('q'),
        KeyModifiers::NONE,
        Generic::Quit.into(),
    );
    key_event_map.add_static(
        Global,
        KeyCode::Char('1'),
        KeyModifiers::NONE,
        Generic::SetFocus(AppFocus::List).into(),
    );
    key_event_map.add_static(
        Global,
        KeyCode::Char('2'),
        KeyModifiers::NONE,
        Generic::SetFocus(AppFocus::Config).into(),
    );
    key_event_map.add_static(
        Global,
        KeyCode::Char('3'),
        KeyModifiers::NONE,
        App::OpenLogs.into(),
    );
    key_event_map.add_static(
        Global,
        KeyCode::Char('c'),
        KeyModifiers::NONE,
        Generic::CopyToClipboard.into(),
    );
    key_event_map.add_static(
        Global,
        KeyCode::Char('o'),
        KeyModifiers::NONE,
        Generic::OpenInBrowser.into(),
    );
    key_event_map.add_static(
        Popup,
        KeyCode::Char('d'),
        KeyModifiers::NONE,
        App::DismissPopup.into(),
    );

    // CONFIG EVENTS
    key_event_map.add_static(
        Config,
        KeyCode::Down,
        KeyModifiers::NONE,
        App::ConfigListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        Config,
        KeyCode::Up,
        KeyModifiers::NONE,
        App::ConfigListMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        Config,
        KeyCode::Enter,
        KeyModifiers::NONE,
        App::ToggleFeature.into(),
    );
    key_event_map.add_static(
        Config,
        KeyCode::Right,
        KeyModifiers::NONE,
        App::OpenToolConfig(Tool::ServiceStatus).into(),
    );
    key_event_map.add_static(
        Config,
        KeyCode::Left,
        KeyModifiers::NONE,
        Generic::SetFocus(AppFocus::List).into(),
    );

    // LOGS EVENTS
    key_event_map.add_static(
        Logs,
        KeyCode::Down,
        KeyModifiers::NONE,
        App::LogsListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        Logs,
        KeyCode::Up,
        KeyModifiers::NONE,
        App::LogsListMove(Direction::Up).into(),
    );

    // TOOL CONFIG EVENTS (Service Status)
    key_event_map.add_static(
        ToolConfig(Tool::ServiceStatus),
        KeyCode::Down,
        KeyModifiers::NONE,
        ServiceStatusConfig::ListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::ServiceStatus),
        KeyCode::Up,
        KeyModifiers::NONE,
        ServiceStatusConfig::ListMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::ServiceStatus),
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        ServiceStatusConfig::OpenAddService.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::ServiceStatus),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        ServiceStatusConfig::OpenEditService.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::ServiceStatus),
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        ServiceStatusConfig::RemoveService.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::ServiceStatus),
        KeyCode::Left,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::ServiceStatus),
        KeyCode::Esc,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );

    // SERVICE STATUS ADD POPUP EVENTS
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Esc,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Enter,
        KeyModifiers::NONE,
        ServiceStatusConfig::SubmitConfig.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormBackspace.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Left,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormLeft.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Right,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormRight.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Home,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormHome.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::End,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormEnd.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Delete,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormDelete.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Down,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormNextField.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Up,
        KeyModifiers::NONE,
        ServiceStatusConfig::PrevField.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::Tab,
        KeyModifiers::NONE,
        ServiceStatusConfig::FormNextField.into(),
    );
    key_event_map.add_static(
        Editing(Tool::ServiceStatus),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        ServiceStatusConfig::PrevField.into(),
    );
    key_event_map.add_dynamic(Editing(Tool::ServiceStatus), service_status_form_char);

    // POP UP EVENTS
    key_event_map.add_static(
        Editing(Tool::Jira),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        Jira::RemoveTicketIdChar.into(),
    );
    key_event_map.add_static(
        Editing(Tool::Jira),
        KeyCode::Left,
        KeyModifiers::NONE,
        Jira::TicketIdLeft.into(),
    );
    key_event_map.add_static(
        Editing(Tool::Jira),
        KeyCode::Right,
        KeyModifiers::NONE,
        Jira::TicketIdRight.into(),
    );
    key_event_map.add_static(
        Editing(Tool::Jira),
        KeyCode::Home,
        KeyModifiers::NONE,
        Jira::TicketIdHome.into(),
    );
    key_event_map.add_static(
        Editing(Tool::Jira),
        KeyCode::End,
        KeyModifiers::NONE,
        Jira::TicketIdEnd.into(),
    );
    key_event_map.add_static(
        Editing(Tool::Jira),
        KeyCode::Delete,
        KeyModifiers::NONE,
        Jira::TicketIdDelete.into(),
    );
    key_event_map.add_static(
        Editing(Tool::Jira),
        KeyCode::Enter,
        KeyModifiers::NONE,
        Jira::SubmitTicketId.into(),
    );
    key_event_map.add_dynamic(Editing(Tool::Jira), add_ticket_id_char);

    // LIST EVENTS
    key_event_map.add_static(
        List,
        KeyCode::Right,
        KeyModifiers::NONE,
        Generic::SetFocus(AppFocus::Tool).into(),
    );
    key_event_map.add_static(
        List,
        KeyCode::Down,
        KeyModifiers::NONE,
        App::ListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        List,
        KeyCode::Up,
        KeyModifiers::NONE,
        App::ListMove(Direction::Up).into(),
    );

    // SERVICE STATUS EVENTS
    key_event_map.add_static(
        ToolCtx(Tool::ServiceStatus),
        KeyCode::Down,
        KeyModifiers::NONE,
        ServiceStatus::ListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::ServiceStatus),
        KeyCode::Up,
        KeyModifiers::NONE,
        ServiceStatus::ListMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::ServiceStatus),
        KeyCode::Char('s'),
        KeyModifiers::NONE,
        ServiceStatus::Scan.into(),
    );

    // TOKEN GENERATOR EVENTS
    key_event_map.add_static(
        ToolIgnore(Tool::TokenGenerator),
        KeyCode::Left,
        KeyModifiers::NONE,
        Generic::SetFocus(AppFocus::List).into(),
    );
    key_event_map.add_static(
        TokenGenCtx(Focus::Service),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGen::ServiceListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        TokenGenCtx(Focus::Service),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGen::ServiceListMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        TokenGenCtx(Focus::Env),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGen::EnvListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        TokenGenCtx(Focus::Env),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGen::EnvListMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::TokenGenerator),
        KeyCode::Right,
        KeyModifiers::NONE,
        TokenGen::SetFocus(Focus::Env).into(),
    );
    key_event_map.add_static(
        TokenGenCtx(Focus::Service),
        KeyCode::Left,
        KeyModifiers::NONE,
        Generic::SetFocus(AppFocus::List).into(),
    );
    key_event_map.add_static(
        TokenGenCtx(Focus::Env),
        KeyCode::Left,
        KeyModifiers::NONE,
        TokenGen::SetFocus(Focus::Service).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::TokenGenerator),
        KeyCode::Enter,
        KeyModifiers::NONE,
        TokenGen::GenerateToken.into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::TokenGenerator),
        KeyCode::Char('c'),
        KeyModifiers::NONE,
        Generic::CopyToClipboard.into(),
    );

    // TOKEN GENERATOR CONFIG EVENTS
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGenConfig::ConfigListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGenConfig::ConfigListMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        TokenGenConfig::OpenAddService.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        TokenGenConfig::ConfigEdit.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        TokenGenConfig::RemoveService.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Tab,
        KeyModifiers::NONE,
        TokenGenConfig::SwitchFocus.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        TokenGenConfig::SwitchFocus.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Left,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::TokenGenerator),
        KeyCode::Esc,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );

    // TOKEN GENERATOR CONFIG POPUP EVENTS
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Esc,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Enter,
        KeyModifiers::NONE,
        TokenGenConfig::SubmitConfig.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        TokenGenConfig::FormBackspace.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Left,
        KeyModifiers::NONE,
        TokenGenConfig::FormLeft.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Right,
        KeyModifiers::NONE,
        TokenGenConfig::FormRight.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Home,
        KeyModifiers::NONE,
        TokenGenConfig::FormHome.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::End,
        KeyModifiers::NONE,
        TokenGenConfig::FormEnd.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Delete,
        KeyModifiers::NONE,
        TokenGenConfig::FormDelete.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Down,
        KeyModifiers::NONE,
        TokenGenConfig::FormNextField.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Up,
        KeyModifiers::NONE,
        TokenGenConfig::FormPrevField.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::Tab,
        KeyModifiers::NONE,
        TokenGenConfig::FormNextField.into(),
    );
    key_event_map.add_static(
        Editing(Tool::TokenGenerator),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        TokenGenConfig::FormPrevField.into(),
    );
    key_event_map.add_dynamic(Editing(Tool::TokenGenerator), token_gen_config_form_char);

    // JIRA EVENTS
    key_event_map.add_static(
        ToolCtx(Tool::Jira),
        KeyCode::Up,
        KeyModifiers::NONE,
        Jira::ListMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::Jira),
        KeyCode::Down,
        KeyModifiers::NONE,
        Jira::ListMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::Jira),
        KeyCode::Up,
        KeyModifiers::SHIFT,
        Jira::TicketMove(Direction::Up).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::Jira),
        KeyCode::Down,
        KeyModifiers::SHIFT,
        Jira::TicketMove(Direction::Down).into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::Jira),
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        Jira::NewTicket.into(),
    );
    key_event_map.add_static(
        ToolCtx(Tool::Jira),
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        Jira::RemoveTicket.into(),
    );

    // JIRA CONFIG EVENTS
    key_event_map.add_static(
        ToolConfig(Tool::Jira),
        KeyCode::Char('e'),
        KeyModifiers::NONE,
        JiraConfig::OpenEdit.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::Jira),
        KeyCode::Left,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );
    key_event_map.add_static(
        ToolConfig(Tool::Jira),
        KeyCode::Esc,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );

    // JIRA CONFIG POPUP EVENTS
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Esc,
        KeyModifiers::NONE,
        App::CloseToolConfig.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Enter,
        KeyModifiers::NONE,
        JiraConfig::SubmitConfig.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Backspace,
        KeyModifiers::NONE,
        JiraConfig::FormBackspace.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Left,
        KeyModifiers::NONE,
        JiraConfig::FormLeft.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Right,
        KeyModifiers::NONE,
        JiraConfig::FormRight.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Home,
        KeyModifiers::NONE,
        JiraConfig::FormHome.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::End,
        KeyModifiers::NONE,
        JiraConfig::FormEnd.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Delete,
        KeyModifiers::NONE,
        JiraConfig::FormDelete.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Down,
        KeyModifiers::NONE,
        JiraConfig::FormNextField.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Up,
        KeyModifiers::NONE,
        JiraConfig::FormPrevField.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::Tab,
        KeyModifiers::NONE,
        JiraConfig::FormNextField.into(),
    );
    key_event_map.add_static(
        ToolConfigEditing(Tool::Jira),
        KeyCode::BackTab,
        KeyModifiers::SHIFT,
        JiraConfig::FormPrevField.into(),
    );
    key_event_map.add_dynamic(ToolConfigEditing(Tool::Jira), jira_config_form_char);
}

fn add_ticket_id_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| Jira::AddTicketIdChar(c).into())
}

fn service_status_form_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| ServiceStatusConfig::FormChar(c).into())
}

fn token_gen_config_form_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| TokenGenConfig::FormChar(c).into())
}

fn jira_config_form_char(key_event: KeyEvent) -> Option<Event> {
    key_event
        .code
        .as_char()
        .map(|c| JiraConfig::FormChar(c).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::events::Direction::{Down, Up};
    use crate::input::key_context::KeyContext;
    use crate::input::key_context::KeyContext::{
        Config, Editing, Global, List, Logs, TokenGen as TokenGenCtx, Tool as ToolCtx, ToolConfig,
        ToolIgnore,
    };
    use crate::state::token_generator::Focus;
    use test_case::test_case;

    fn registered_map() -> KeyEventMap {
        let mut map = KeyEventMap::default();
        register_bindings(&mut map);
        map
    }

    #[test_case(Global, KeyCode::Char('q'), KeyModifiers::NONE, Generic::Quit.into(); "q quits")]
    #[test_case(Global, KeyCode::Char('1'), KeyModifiers::NONE, Generic::SetFocus(AppFocus::List).into(); "1 focuses tools list")]
    #[test_case(Global, KeyCode::Char('2'), KeyModifiers::NONE, Generic::SetFocus(AppFocus::Config).into(); "2 focuses config")]
    #[test_case(Global, KeyCode::Char('3'), KeyModifiers::NONE, App::OpenLogs.into(); "3 opens logs")]
    #[test_case(Global, KeyCode::Char('c'), KeyModifiers::NONE, Generic::CopyToClipboard.into(); "c copies")]
    #[test_case(Global, KeyCode::Char('o'), KeyModifiers::NONE, Generic::OpenInBrowser.into(); "o opens browser")]
    #[test_case(KeyContext::Popup, KeyCode::Char('d'), KeyModifiers::NONE, App::DismissPopup.into(); "popup dismissed")]
    #[test_case(Config, KeyCode::Down, KeyModifiers::NONE, App::ConfigListMove(Down).into(); "config down")]
    #[test_case(Config, KeyCode::Up, KeyModifiers::NONE, App::ConfigListMove(Up).into(); "config up")]
    #[test_case(Config, KeyCode::Enter, KeyModifiers::NONE, App::ToggleFeature.into(); "config enter toggles feature")]
    #[test_case(Config, KeyCode::Left, KeyModifiers::NONE, Generic::SetFocus(AppFocus::List).into(); "config left focuses tools list")]
    #[test_case(Config, KeyCode::Right, KeyModifiers::NONE, App::OpenToolConfig(Tool::ServiceStatus).into(); "config right opens tool config")]
    #[test_case(Logs, KeyCode::Down, KeyModifiers::NONE, App::LogsListMove(Down).into(); "logs down navigates")]
    #[test_case(Logs, KeyCode::Up, KeyModifiers::NONE, App::LogsListMove(Up).into(); "logs up navigates")]
    #[test_case(ToolConfig(Tool::ServiceStatus), KeyCode::Down, KeyModifiers::NONE, ServiceStatusConfig::ListMove(Down).into(); "tool config down")]
    #[test_case(ToolConfig(Tool::ServiceStatus), KeyCode::Up, KeyModifiers::NONE, ServiceStatusConfig::ListMove(Up).into(); "tool config up")]
    #[test_case(ToolConfig(Tool::ServiceStatus), KeyCode::Char('a'), KeyModifiers::NONE, ServiceStatusConfig::OpenAddService.into(); "tool config a opens add form")]
    #[test_case(ToolConfig(Tool::ServiceStatus), KeyCode::Char('x'), KeyModifiers::NONE, ServiceStatusConfig::RemoveService.into(); "tool config x removes service")]
    #[test_case(ToolConfig(Tool::ServiceStatus), KeyCode::Left, KeyModifiers::NONE, App::CloseToolConfig.into(); "tool config left closes")]
    #[test_case(Editing(Tool::ServiceStatus), KeyCode::Enter, KeyModifiers::NONE, ServiceStatusConfig::SubmitConfig.into(); "service form enter submits")]
    #[test_case(Editing(Tool::ServiceStatus), KeyCode::Backspace, KeyModifiers::NONE, ServiceStatusConfig::FormBackspace.into(); "service form backspace")]
    #[test_case(Editing(Tool::ServiceStatus), KeyCode::Tab, KeyModifiers::NONE, ServiceStatusConfig::FormNextField.into(); "service form tab next field")]
    #[test_case(Editing(Tool::ServiceStatus), KeyCode::BackTab, KeyModifiers::SHIFT, ServiceStatusConfig::PrevField.into(); "service form shift-tab prev field")]
    #[test_case(List, KeyCode::Right, KeyModifiers::NONE, Generic::SetFocus(AppFocus::Tool).into(); "list right focuses tool")]
    #[test_case(List, KeyCode::Down, KeyModifiers::NONE, App::ListMove(Down).into(); "list down")]
    #[test_case(List, KeyCode::Up, KeyModifiers::NONE, App::ListMove(Up).into(); "list up")]
    #[test_case(ToolCtx(Tool::ServiceStatus), KeyCode::Down, KeyModifiers::NONE, ServiceStatus::ListMove(Down).into(); "service status down")]
    #[test_case(ToolCtx(Tool::ServiceStatus), KeyCode::Up, KeyModifiers::NONE, ServiceStatus::ListMove(Up).into(); "service status up")]
    #[test_case(ToolCtx(Tool::ServiceStatus), KeyCode::Char('s'), KeyModifiers::NONE, ServiceStatus::Scan.into(); "s scans services")]
    #[test_case(ToolIgnore(Tool::TokenGenerator), KeyCode::Left, KeyModifiers::NONE, Generic::SetFocus(AppFocus::List).into(); "tool left focuses list")]
    #[test_case(TokenGenCtx(Focus::Service), KeyCode::Down, KeyModifiers::NONE, TokenGen::ServiceListMove(Down).into(); "token service down")]
    #[test_case(TokenGenCtx(Focus::Service), KeyCode::Up, KeyModifiers::NONE, TokenGen::ServiceListMove(Up).into(); "token service up")]
    #[test_case(TokenGenCtx(Focus::Env), KeyCode::Down, KeyModifiers::NONE, TokenGen::EnvListMove(Down).into(); "token env down")]
    #[test_case(TokenGenCtx(Focus::Env), KeyCode::Up, KeyModifiers::NONE, TokenGen::EnvListMove(Up).into(); "token env up")]
    #[test_case(ToolCtx(Tool::TokenGenerator), KeyCode::Right, KeyModifiers::NONE, TokenGen::SetFocus(Focus::Env).into(); "token right focuses env")]
    #[test_case(TokenGenCtx(Focus::Service), KeyCode::Left, KeyModifiers::NONE, Generic::SetFocus(AppFocus::List).into(); "token service left focuses list")]
    #[test_case(TokenGenCtx(Focus::Env), KeyCode::Left, KeyModifiers::NONE, TokenGen::SetFocus(Focus::Service).into(); "token env left focuses service")]
    #[test_case(ToolCtx(Tool::TokenGenerator), KeyCode::Enter, KeyModifiers::NONE, TokenGen::GenerateToken.into(); "token enter generates")]
    #[test_case(ToolCtx(Tool::Jira), KeyCode::Up, KeyModifiers::NONE, Jira::ListMove(Up).into(); "jira up")]
    #[test_case(ToolCtx(Tool::Jira), KeyCode::Down, KeyModifiers::NONE, Jira::ListMove(Down).into(); "jira down")]
    #[test_case(ToolCtx(Tool::Jira), KeyCode::Up, KeyModifiers::SHIFT, Jira::TicketMove(Up).into(); "jira shift up moves ticket")]
    #[test_case(ToolCtx(Tool::Jira), KeyCode::Down, KeyModifiers::SHIFT, Jira::TicketMove(Down).into(); "jira shift down moves ticket")]
    #[test_case(ToolCtx(Tool::Jira), KeyCode::Char('a'), KeyModifiers::NONE, Jira::NewTicket.into(); "jira a adds ticket")]
    #[test_case(ToolCtx(Tool::Jira), KeyCode::Char('x'), KeyModifiers::NONE, Jira::RemoveTicket.into(); "jira x removes ticket")]
    #[test_case(Editing(Tool::Jira), KeyCode::Backspace, KeyModifiers::NONE, Jira::RemoveTicketIdChar.into(); "form backspace removes char")]
    #[test_case(Editing(Tool::Jira), KeyCode::Enter, KeyModifiers::NONE, Jira::SubmitTicketId.into(); "form enter submits")]
    fn binding_resolves_to_expected_event(
        context: KeyContext,
        code: KeyCode,
        modifiers: KeyModifiers,
        expected: Event,
    ) {
        let map = registered_map();
        let result = map.resolve(context, KeyEvent::new(code, modifiers));
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn popup_dynamic_handler_maps_char_to_add_ticket_id_char() {
        let map = registered_map();
        let result = map.resolve(
            Editing(Tool::Jira),
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE),
        );
        assert_eq!(result, Some(Jira::AddTicketIdChar('A').into()));
    }

    #[test]
    fn popup_dynamic_handler_returns_none_for_non_char() {
        let map = registered_map();
        let result = map.resolve(
            Editing(Tool::Jira),
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        );
        assert_eq!(result, None);
    }
}
