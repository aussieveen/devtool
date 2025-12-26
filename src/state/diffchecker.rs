use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use ratatui::widgets::ListState;
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use crate::config::{DiffChecker as DiffCheckerConfig};
use crate::events::event::AppEvent;
use crate::events::sender::EventSender;
use crate::state::diffchecker::Commit::Fetching;
use crate::state::diffchecker::LinkStatus::{Diff, Errored, Missing, NoDiff};

#[derive(Debug)]
pub struct DiffChecker {
    pub services: Vec<Service>,
    pub list_state: ListState,
    pub event_sender: EventSender
}

#[derive(Debug, PartialEq)]
pub enum Commit{
    NotFetched,
    Fetching,
    Fetched(String),
    Error(String)
}

impl Commit{
    fn fetched_value(&self) -> Option<&str> {
        match self {
            Commit::Fetched(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl DiffChecker {
    pub fn new(config: Vec<DiffCheckerConfig>, event_sender: EventSender) -> Self {
        Self {
            services: config.into_iter().map(Service::new).collect(),
            list_state: ListState::default().with_selected(Some(0)),
            event_sender
        }
    }

    pub(crate) async fn set_preprod_commit(&mut self, service_idx: usize) {
        self.services[service_idx].preprod = Commit::Fetching;
        let url = self.services[service_idx].config.preprod.clone();
        let sender = self.event_sender.clone();
        tokio::spawn(async move {
            let commit = match Self::get_commit_from_healthcheck(&url).await {
                Ok(commit) => Commit::Fetched(commit),
                Err(err) => {
                    println!("{}", err.to_string());
                    Commit::Error(err.to_string())
                }
            };

            sender.send(AppEvent::PreprodCommit(commit, service_idx))
        });
    }

    pub(crate) async fn set_prod_commit(&mut self, service_idx: usize) {
        self.services[service_idx].prod = Commit::Fetching;
        let url = self.services[service_idx].config.prod.clone();
        let sender = self.event_sender.clone();
        tokio::spawn(async move {
            let commit = match Self::get_commit_from_healthcheck(&url).await {
                Ok(commit) => Commit::Fetched(commit),
                Err(err) => {
                    println!("{}", err.to_string());
                    Commit::Error(err.to_string())
                }
            };

            sender.send(AppEvent::ProdCommit(commit, service_idx))
        });
    }

    pub(crate) fn get_link(&self, service_idx: usize) -> String {
        let service = &self.services[service_idx];
        format!(
            "{}/compare/{}...{}",
            service.config.repo,
            service.prod.fetched_value().unwrap(),
            service.preprod.fetched_value().unwrap(),
        )
    }

    async fn get_commit_from_healthcheck(base_url: &String) -> Result<String, Box<dyn Error>> {
        let mut url = base_url.clone();
        url.push_str("healthcheck");

        let client = reqwest::Client::new();
        let resp = client.get(url)
            .header(USER_AGENT, "chrome")
            .header(ACCEPT, "application/json")
            .timeout(Duration::from_secs(3))
            .send()
            .await?;

        Ok(resp.json::<Healthcheck>().await?.version.split("_").collect())
    }
}

#[derive(Deserialize)]
#[derive(Debug)]
struct Healthcheck {
    version: String
}

#[derive(Debug)]
pub struct Service {
    pub config: DiffCheckerConfig,
    pub preprod: Commit,
    pub prod: Commit
}

pub enum LinkStatus {
    Missing,
    Fetching,
    Errored,
    NoDiff,
    Diff
}

impl Service {
    pub fn new(config: DiffCheckerConfig) -> Self {
        Self {
            config,
            preprod: Commit::NotFetched,
            prod: Commit::NotFetched
        }
    }

    pub fn preprod_fetched(&self) -> bool{
        matches!(self.preprod, Commit::Fetched(_))
    }
    pub fn preprod_errored(&self) -> bool { matches!(self.preprod, Commit::Error(_))}

    pub fn prod_fetched(&self) -> bool{
        matches!(self.preprod, Commit::Fetched(_))
    }
    pub fn prod_errored(&self) -> bool { matches!(self.prod, Commit::Error(_))}

    pub fn prod_fetching(&self) -> bool { matches!(self.prod, Commit::Fetching)}
    pub fn preprod_fetching(&self) -> bool { matches!(self.preprod, Commit::Fetching)}

    pub fn link_status(&self) -> LinkStatus{
        if self.preprod_errored() || self.prod_errored() {
            return LinkStatus::Errored;
        }

        if self.preprod_fetching() || self.prod_fetching() {
            return LinkStatus::Fetching;
        }

        if !self.preprod_fetched() || !self.prod_fetched() {
            return LinkStatus::Missing;
        }

        if self.prod.fetched_value().unwrap() == self.preprod.fetched_value().unwrap(){
            return LinkStatus::NoDiff;
        }

        LinkStatus::Diff

    }
}