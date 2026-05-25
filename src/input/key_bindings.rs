use crate::app::AppFocus;
use crate::event::events::{AppEvent as App, Direction, GenericEvent as Generic};
use crate::input::key_context::KeyContext::{Config, Global, List, Logs, Popup};
use crate::input::key_event_map::KeyEventMap;
use crossterm::event::{KeyCode, KeyModifiers};

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
        App::OpenToolConfig.into(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::events::Direction::{Down, Up};
    use crate::event::events::Event;
    use crate::input::key_context::KeyContext;
    use crossterm::event::KeyEvent;
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
    #[test_case(Config, KeyCode::Right, KeyModifiers::NONE, App::OpenToolConfig.into(); "config right opens tool config")]
    #[test_case(Logs, KeyCode::Down, KeyModifiers::NONE, App::LogsListMove(Down).into(); "logs down navigates")]
    #[test_case(Logs, KeyCode::Up, KeyModifiers::NONE, App::LogsListMove(Up).into(); "logs up navigates")]
    #[test_case(List, KeyCode::Right, KeyModifiers::NONE, Generic::SetFocus(AppFocus::Tool).into(); "list right focuses tool")]
    #[test_case(List, KeyCode::Down, KeyModifiers::NONE, App::ListMove(Down).into(); "list down")]
    #[test_case(List, KeyCode::Up, KeyModifiers::NONE, App::ListMove(Up).into(); "list up")]
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
}
