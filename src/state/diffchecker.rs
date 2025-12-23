use ratatui::widgets::ListState;
use crate::config::{DiffChecker as DiffCheckerConfig};

#[derive(Debug)]
pub struct DiffChecker {
    pub services: Vec<Service>,
    pub state: ListState
}

impl DiffChecker {
    pub fn new(config: Vec<DiffCheckerConfig>) -> Self {
        Self {
            services: config.into_iter().map(Service::new).collect(),
            state: ListState::default().with_selected(Some(0))
        }
    }
}

#[derive(Debug)]
pub struct Service {
    pub config: DiffCheckerConfig,
    pub preprod_commit: Option<String>,
    pub prod_commit: Option<String>
}

impl Service {
    pub fn new(config: DiffCheckerConfig) -> Self {
        Self {
            config,
            preprod_commit: None,
            prod_commit: None,
        }
    }
}