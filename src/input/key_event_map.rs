use crate::events::event::AppEvent;
use crate::input::key_context::KeyContext;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

type Key = (KeyCode, KeyModifiers);
type KeyHandler = fn(KeyEvent) -> Option<AppEvent>;
#[derive(Debug)]
pub struct KeyEventMap {
    static_events: HashMap<(KeyContext, Key), AppEvent>,
    dynamic_events: HashMap<KeyContext, KeyHandler>,
}

impl KeyEventMap {
    pub fn new() -> KeyEventMap {
        KeyEventMap {
            static_events: HashMap::new(),
            dynamic_events: HashMap::new(),
        }
    }

    pub fn add_static(
        &mut self,
        context: KeyContext,
        key_code: KeyCode,
        key_modifiers: KeyModifiers,
        event: AppEvent,
    ) {
        self.static_events
            .insert((context, (key_code, key_modifiers)), event);
    }

    pub fn add_dynamic(&mut self, context: KeyContext, function: KeyHandler) {
        self.dynamic_events.insert(context, function);
    }

    pub fn resolve(&mut self, context: KeyContext, key: KeyEvent) -> Option<AppEvent> {
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
