use std::collections::HashMap;
use std::error::Error;
use ratatui::widgets::ListState;
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use crate::config::{DiffChecker as DiffCheckerConfig};
use crate::state::diffchecker::LinkStatus::{Diff, Missing, NoDiff};

#[derive(Debug)]
pub struct DiffChecker {
    pub services: Vec<Service>,
    pub list_state: ListState
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
    pub fn new(config: Vec<DiffCheckerConfig>) -> Self {
        Self {
            services: config.into_iter().map(Service::new).collect(),
            list_state: ListState::default().with_selected(Some(0))
        }
    }

     pub(crate) async fn set_preprod_commit(&mut self, service_idx: usize) {
        let service = &mut self.services[service_idx];

        service.preprod = Commit::Fetching;
        service.preprod = Commit::Fetched(
            Self::get_commit_from_healthcheck(&service.config.preprod).await.unwrap()
        );
    }

    pub(crate) async fn set_prod_commit(&mut self, service_idx: usize) {
        let service = &mut self.services[service_idx];

        service.prod = Commit::Fetching;
        service.prod = Commit::Fetched(
            Self::get_commit_from_healthcheck(&service.config.prod).await.unwrap()
        );
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

    pub fn prod_fetched(&self) -> bool{
        matches!(self.preprod, Commit::Fetched(_))
    }

    pub fn link_status(&self) -> LinkStatus{
        if !self.preprod_fetched() || !self.prod_fetched() {
            return Missing;
        }
        
        if self.prod.fetched_value().unwrap() == self.preprod.fetched_value().unwrap(){
            return NoDiff;
        }
        
        Diff
        
    }
}