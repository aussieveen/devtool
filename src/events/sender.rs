use tokio::sync::mpsc;
use crate::events::event::{AppEvent, Event};

#[derive(Clone, Debug)]
pub struct EventSender {
    pub(crate) sender: mpsc::UnboundedSender<Event>,
}

impl EventSender {
    pub fn send(&self, app_event: AppEvent) {
        let _ = self.sender.send(Event::App(app_event));
    }
}
