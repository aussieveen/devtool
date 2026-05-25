use crate::event::events::Event;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct EventSender {
    pub(crate) sender: mpsc::Sender<Event>,
}

impl EventSender {
    pub fn send<E: Into<Event>>(&self, event: E) {
        let _ = self.sender.try_send(event.into());
    }
}
