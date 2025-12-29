use color_eyre::eyre::OptionExt;
use tokio::sync::mpsc;
use crate::events::event::Event;
use crate::events::sender::EventSender;
use crate::events::task::EventTask;

#[derive(Debug)]
pub struct EventHandler {
    sender: EventSender,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let sender = EventSender { sender: tx.clone() };

        let actor = EventTask::new(tx.clone());
        
        tokio::spawn(async { actor.run().await });

        Self {
            sender,
            receiver: rx,
        }
    }

    pub fn sender(&self) -> EventSender {
        self.sender.clone()
    }

    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }
}