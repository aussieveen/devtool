use crate::config::ServiceStatus as ServiceStatusConfig;
use crate::environment::Environment;
use crate::events::event::AppEvent;
use crate::events::sender::EventSender;
use ratatui::widgets::ListState;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum Commit {
    NotFetched,
    Fetching,
    Fetched(String),
    Error(String),
}

impl Commit {
    pub fn value(&self) -> Option<&str> {
        match self {
            Commit::Fetched(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn short_value(&self) -> Option<String> {
        let char_display_count = 6;
        match self {
            Commit::Fetched(s) => {
                let first: String = s.chars().take(char_display_count).collect();
                let last: String = s
                    .chars()
                    .rev()
                    .take(char_display_count)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                Some(format!("{}...{}", first, last))
            }
            _ => None,
        }
    }

    fn is_errored(&self) -> bool {
        matches!(self, Commit::Error(_))
    }
}

#[derive(Debug)]
pub struct ServiceStatus {
    pub services: Vec<Service>,
    pub list_state: ListState,
    pub event_sender: EventSender,
}

impl ServiceStatus {
    pub fn new(config: Vec<ServiceStatusConfig>, event_sender: EventSender) -> Self {
        Self {
            services: config.into_iter().map(Service::new).collect(),
            list_state: ListState::default().with_selected(None),
            event_sender,
        }
    }

    pub(crate) async fn set_commit(&mut self, service_idx: usize, env: Environment) {
        let url = match env {
            Environment::Staging => {
                self.services[service_idx].staging = Commit::Fetching;
                self.services[service_idx].staging_url.clone()
            }
            Environment::Preproduction => {
                self.services[service_idx].preprod = Commit::Fetching;
                self.services[service_idx].preprod_url.clone()
            }
            Environment::Production => {
                self.services[service_idx].prod = Commit::Fetching;
                self.services[service_idx].prod_url.clone()
            }
            _ => String::from(""),
        };

        let sender = self.event_sender.clone();

        tokio::spawn(async move {
            let commit = match Self::get_commit_from_healthcheck(&url).await {
                Ok(commit) => Commit::Fetched(commit),
                Err(err) => Commit::Error(err.to_string()),
            };

            sender.send(AppEvent::CommitRefRetrieved(commit, service_idx, env))
        });
    }

    pub(crate) fn has_link(&self) -> bool {
        let service = &self.services[self.list_state.selected().unwrap()];
        service.commit_ref_status() == CommitRefStatus::StagingPreprodMatch
    }

    pub(crate) fn get_link(&self) -> String {
        let service = &self.services[self.list_state.selected().unwrap()];
        format!(
            "{}compare/{}...{}",
            service.repo_url,
            service.prod.value().unwrap(),
            service.preprod.value().unwrap(),
        )
    }

    async fn get_commit_from_healthcheck(base_url: &str) -> Result<String, Box<dyn Error>> {
        let mut url = base_url.to_owned();
        url.push_str("healthcheck");

        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .header(USER_AGENT, "chrome")
            .header(ACCEPT, "application/json")
            .timeout(Duration::from_secs(3))
            .send()
            .await?;

        Ok(resp
            .json::<Healthcheck>()
            .await?
            .version
            .split("_")
            .collect())
    }
}

#[derive(Deserialize, Debug)]
struct Healthcheck {
    version: String,
}

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub staging_url: String,
    pub preprod_url: String,
    pub prod_url: String,
    pub repo_url: String,
    pub staging: Commit,
    pub preprod: Commit,
    pub prod: Commit,
}

#[derive(PartialEq)]
pub enum CommitRefStatus {
    NothingMatches,
    AllMatches,
    StagingPreprodMatch,
    PreprodProdMatch,
    CommitMissing,
}

impl Service {
    pub fn new(config: ServiceStatusConfig) -> Self {
        Self {
            name: config.name,
            staging_url: config.staging,
            preprod_url: config.preprod,
            prod_url: config.prod,
            repo_url: config.repo,
            staging: Commit::NotFetched,
            preprod: Commit::NotFetched,
            prod: Commit::NotFetched,
        }
    }

    pub fn commit_ref_status(&self) -> CommitRefStatus {
        if self.prod.is_errored() || self.preprod.is_errored() || self.staging.is_errored() {
            return CommitRefStatus::CommitMissing;
        }

        let preprod_prod_match = self.prod.value() == self.preprod.value();
        let staging_preprod_match = self.preprod.value() == self.staging.value();

        if preprod_prod_match && staging_preprod_match {
            return CommitRefStatus::AllMatches;
        }

        if preprod_prod_match {
            return CommitRefStatus::PreprodProdMatch;
        }

        if staging_preprod_match {
            return CommitRefStatus::StagingPreprodMatch;
        }

        CommitRefStatus::NothingMatches
    }
}
