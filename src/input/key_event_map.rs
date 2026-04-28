use crate::event::event::{AppEvent, Event};
use crate::input::key_context::KeyContext;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

type Key = (KeyCode, KeyModifiers);
type KeyHandler = fn(KeyEvent) -> Option<Event>;

#[derive(Default)]
pub struct KeyEventMap {
    static_events: HashMap<(KeyContext, Key), Event>,
    dynamic_events: HashMap<KeyContext, KeyHandler>,
}

impl KeyEventMap {
    pub fn add_static(
        &mut self,
        context: KeyContext,
        key_code: KeyCode,
        key_modifiers: KeyModifiers,
        event: Event,
    ) {
        self.static_events
            .insert((context, (key_code, key_modifiers)), event);
    }

    pub fn add_dynamic(&mut self, context: KeyContext, function: KeyHandler) {
        self.dynamic_events.insert(context, function);
    }

    pub fn resolve(&self, context: KeyContext, key: KeyEvent) -> Option<Event> {
        let event = self
            .static_events
            .get(&(context.clone(), (key.code, key.modifiers)))
            .cloned();
        if event.is_some() {
            return event;
        }
        if let Some(function) = self.dynamic_events.get(&(context)) {
            return function(key);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::event::{Direction, TokenGeneratorEvent};
    use crate::event::event::AppEvent;

    #[test]
    fn map_add_static() {
        let mut map = KeyEventMap::default();

        map.add_static(
            KeyContext::Global,
            KeyCode::Up,
            KeyModifiers::SHIFT,
            Event::App(AppEvent::ListMove(Direction::Up)),
        );

        assert_eq!(
            map.static_events
                .get(&(KeyContext::Global, (KeyCode::Up, KeyModifiers::SHIFT))),
            Some(&Event::App(AppEvent::ListMove(Direction::Up)))
        );
    }

    #[test]
    fn map_add_dynamic() {
        let mut map = KeyEventMap::default();

        map.add_dynamic(KeyContext::Global, dynamic_function);

        let saved_dynamic_event = map.dynamic_events.get(&KeyContext::Global).unwrap();

        assert_eq!(
            saved_dynamic_event(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)),
            Some(Event::TokenGenerator(TokenGeneratorEvent::GenerateToken))
        );
    }

    #[test]
    fn map_resolve() {
        let mut map = KeyEventMap::default();
        map.add_static(
            KeyContext::Global,
            KeyCode::Up,
            KeyModifiers::SHIFT,
            Event::App(AppEvent::ListMove(Direction::Up)),
        );
        map.add_dynamic(KeyContext::Global, dynamic_function);

        assert_eq!(
            map.resolve(
                KeyContext::Global,
                KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)
            ),
            Some(Event::App(AppEvent::ListMove(Direction::Up)))
        );

        assert_eq!(
            map.resolve(
                KeyContext::Global,
                KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT)
            ),
            Some(Event::TokenGenerator(TokenGeneratorEvent::GenerateToken))
        );

        assert_eq!(
            map.resolve(
                KeyContext::List,
                KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL)
            ),
            None
        );
    }

    fn dynamic_function(_key_event: KeyEvent) -> Option<Event> {
        Some(Event::TokenGenerator(TokenGeneratorEvent::GenerateToken))
    }
}
