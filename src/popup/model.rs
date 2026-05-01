use crate::event::events::Event;
use crate::ui::widgets::popup::{Part, Type};

pub struct Popup {
    pub popup_type: Type,
    pub title: String,
    pub parts: Vec<Part>,
    pub actions: Vec<PopupAction>
}

pub struct PopupAction {
    pub key: char,
    pub label: String,
    pub event: Event,
}

impl Popup {
    pub fn new(popup_type: Type, title:String, parts: Vec<Part>) -> Self {
        Self {
            popup_type,
            title,
            parts,
            actions: vec![],
        }
    }
    pub fn with_action(mut self, key: char, label: &str, event: Event) -> Self
    {
        self.actions.push(PopupAction{ key, label: label.to_string(), event } );
        self
    }
}