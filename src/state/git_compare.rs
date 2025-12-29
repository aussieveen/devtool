use crate::config::GitCompare as GitCompareServiceConfig;
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
    fn value(&self) -> Option<&str> {
        match self {
            Commit::Fetched(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn is_errored(&self) -> bool {
        matches!(self, Commit::Error(_))
    }

    fn is_fetching(&self) -> bool {
        matches!(self, Commit::Fetching)
    }

    fn is_fetched(&self) -> bool {
        matches!(self, Commit::Fetched(_))
    }
}

#[derive(Debug)]
pub struct GitCompare {
    pub services: Vec<Service>,
    pub list_state: ListState,
    pub event_sender: EventSender,
}

impl GitCompare {
    pub fn new(config: Vec<GitCompareServiceConfig>, event_sender: EventSender) -> Self {
        Self {
            services: config.into_iter().map(Service::new).collect(),
            list_state: ListState::default().with_selected(Some(0)),
            event_sender,
        }
    }

    pub(crate) async fn set_commit(&mut self, service_idx: usize, env: Environment) {
        let url = match env {
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
    pub preprod_url: String,
    pub prod_url: String,
    pub repo_url: String,
    pub preprod: Commit,
    pub prod: Commit,
}

pub enum LinkStatus {
    Missing,
    Fetching,
    Errored,
    NoDiff,
    Diff,
}

impl Service {
    pub fn new(config: GitCompareServiceConfig) -> Self {
        Self {
            name: config.name,
            preprod_url: config.preprod,
            prod_url: config.prod,
            repo_url: config.repo,
            preprod: Commit::NotFetched,
            prod: Commit::NotFetched,
        }
    }

    pub fn link_status(&self) -> LinkStatus {
        if self.preprod.is_errored() || self.prod.is_errored() {
            return LinkStatus::Errored;
        }

        if self.preprod.is_fetching() || self.prod.is_fetching() {
            return LinkStatus::Fetching;
        }

        if !self.preprod.is_fetched() || !self.prod.is_fetched() {
            return LinkStatus::Missing;
        }

        if self.prod.value() == self.preprod.value() {
            return LinkStatus::NoDiff;
        }

        LinkStatus::Diff
    }
}
