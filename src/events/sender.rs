use crate::events::event::{AppEvent, Event};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct EventSender {
    pub(crate) sender: mpsc::UnboundedSender<Event>,
}

impl EventSender {
    pub fn send(&self, app_event: AppEvent) {
        let _ = self.sender.send(Event::App(app_event));
    }
}
