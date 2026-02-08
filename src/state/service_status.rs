use crate::environment::Environment;
use ratatui::widgets::ListState;

pub struct ServiceStatus {
    pub services: Vec<Service>,
    pub list_state: ListState,
}

impl ServiceStatus {
    pub fn new(num_services: usize) -> Self {
        Self {
            services: vec![Service::new(); num_services],
            list_state: ListState::default().with_selected(None),
        }
    }

    pub fn set_commit_fetching(&mut self, service_idx: usize, env: &Environment) {
        self.update_commit(service_idx, env, Commit::Fetching);
    }

    pub fn set_commit_ok(&mut self, service_idx: usize, env: &Environment, commit: String) {
        self.update_commit(service_idx, env, Commit::Ok(commit));
    }

    pub fn set_commit_error(&mut self, service_idx: usize, env: &Environment, error: String) {
        self.update_commit(service_idx, env, Commit::Error(error));
    }

    fn update_commit(&mut self, service_idx: usize, env: &Environment, commit: Commit) {
        let service = &mut self.services[service_idx];

        match env {
            Environment::Staging => service.staging = commit,
            Environment::Preproduction => service.preprod = commit,
            Environment::Production => service.prod = commit,
            _ => {}
        }
    }

    pub fn get_selected_service_idx(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub(crate) fn has_link(&self) -> bool {
        let service = &self.services[self.list_state.selected().unwrap()];
        service.commit_ref_status() == CommitRefStatus::StagingPreprodMatch
    }

    pub(crate) fn get_link(&self, repo_url: &String) -> String {
        let service = &self.services[self.list_state.selected().unwrap()];
        format!(
            "{}compare/{}...{}",
            repo_url,
            service.prod.value().unwrap(),
            service.preprod.value().unwrap(),
        )
    }
}

#[derive(PartialEq, Clone)]
pub enum Commit {
    Empty,
    Fetching,
    Ok(String),
    Error(String),
}

impl Commit {
    pub fn value(&self) -> Option<&str> {
        match self {
            Commit::Ok(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn short_value(&self) -> Option<String> {
        let char_display_count = 6;
        match self {
            Commit::Ok(s) => {
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

#[derive(Clone)]
pub struct Service {
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
    pub fn new() -> Self {
        Self {
            staging: Commit::Empty,
            preprod: Commit::Empty,
            prod: Commit::Empty,
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
