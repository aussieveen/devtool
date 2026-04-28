use crate::event::event::{AppEvent, Event, GenericEvent, JiraConfigEvent, JiraEvent, ServiceStatusConfigEvent, ServiceStatusEvent, TokenGeneratorConfigEvent, TokenGeneratorEvent};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct EventSender {
    pub(crate) sender: mpsc::UnboundedSender<Event>,
}

impl EventSender {
    pub fn send_event(&self, event: Event){
        let _ = self.sender.send(event);
    }

    pub fn send_app_event(&self, event: AppEvent) {
        self.send_event(Event::App(event));
    }

    pub fn send_service_status_event(&self, event: ServiceStatusEvent){
        self.send_event(Event::ServiceStatus(event));
    }

    pub fn send_service_status_config_event(&self, event: ServiceStatusConfigEvent){
        self.send_event(Event::ServiceStatusConfig(event));
    }

    pub fn send_token_generator_event(&self, event: TokenGeneratorEvent){
        self.send_event(Event::TokenGenerator(event));
    }

    pub fn send_token_generator_config_event(&self, event: TokenGeneratorConfigEvent){
        self.send_event(Event::TokenGeneratorConfig(event));
    }

    pub fn send_jira_event(&self, event: JiraEvent){
        self.send_event(Event::Jira(event));
    }

    pub fn send_jira_config_event(&self, event: JiraConfigEvent){
        self.send_event(Event::JiraConfig(event));
    }
}
