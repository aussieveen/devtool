use crate::event::event::{AppEvent, Event, JiraConfigEvent, JiraEvent, ServiceStatusConfigEvent, ServiceStatusEvent, TokenGeneratorConfigEvent, TokenGeneratorEvent};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct EventSender {
    pub(crate) sender: mpsc::UnboundedSender<Event>,
}

impl EventSender {
    pub fn send_app_event(&self, event: AppEvent) {
        let _ = self.sender.send(Event::App(event));
    }

    pub fn send_service_status_event(&self, event: ServiceStatusEvent){
        let _ = self.sender.send(Event::ServiceStatus(event));
    }

    pub fn send_service_status_config_event(&self, event: ServiceStatusConfigEvent){
        let _ = self.sender.send(Event::ServiceStatusConfig(event));
    }

    pub fn send_token_generator_event(&self, event: TokenGeneratorEvent){
        let _ = self.sender.send(Event::TokenGenerator(event));
    }

    pub fn send_token_generator_config_event(&self, event: TokenGeneratorConfigEvent){
        let _ = self.sender.send(Event::TokenGeneratorConfig(event));
    }

    pub fn send_jira_event(&self, event: JiraEvent){
        let _ = self.sender.send(Event::Jira(event));
    }

    pub fn send_jira_config_event(&self, event: JiraConfigEvent){
        let _ = self.sender.send(Event::JiraConfig(event));
    }
}
