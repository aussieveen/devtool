use ratatui::widgets::ListState;
use crate::config::{DiffChecker as DiffCheckerConfig};
use crate::events::sender::EventSender;

#[derive(Debug)]
pub struct DiffChecker {
    pub services: Vec<Service>,
    pub state: ListState
}

#[derive(Debug, PartialEq)]
pub enum Commit{
    NotFetched,
    Fetching,
    Fetched(String),
    Error(String)
}

impl DiffChecker {
    pub fn new(config: Vec<DiffCheckerConfig>) -> Self {
        Self {
            services: config.into_iter().map(Service::new).collect(),
            state: ListState::default().with_selected(Some(0))
        }
    }

    pub(crate) fn set_preprod_commit(&mut self, service_idx: usize) {
        self.services[service_idx].preprod = Commit::Fetching;
        self.services[service_idx].preprod = Commit::Fetched("mypreprodstring".to_string());
    }

    pub(crate) fn set_prod_commit(&mut self, service_idx: usize) {
        self.services[service_idx].prod = Commit::Fetching;
        self.services[service_idx].prod = Commit::Fetched("myprodstring".to_string())
    }
}

#[derive(Debug)]
pub struct Service {
    pub config: DiffCheckerConfig,
    pub preprod: Commit,
    pub prod: Commit
}

impl Service {
    pub fn new(config: DiffCheckerConfig) -> Self {
        Self {
            config,
            preprod: Commit::NotFetched,
            prod: Commit::NotFetched,
        }
    }

    pub fn preprod_fetched(&self) -> bool{
        matches!(self.preprod, Commit::Fetched(_))
    }

    pub fn prod_fetched(&self) -> bool{
        matches!(self.preprod, Commit::Fetched(_))
    }
}