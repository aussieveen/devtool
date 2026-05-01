use crate::ui::widgets::popup::{Part, Type};

pub struct Popup {
    pub popup_type: Type,
    pub title: String,
    pub parts: Vec<Part>
}

impl Popup {
    pub fn new(popup_type: Type, title:String, parts: Vec<Part>) -> Self {
        Self {
            popup_type,
            title,
            parts
        }
    }
}