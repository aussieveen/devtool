use crate::environment::Environment;
use ratatui::widgets::TableState;

pub struct ServiceStatus {
    pub services: Vec<Service>,
    pub table_state: TableState,
}

impl ServiceStatus {
    pub fn new(num_services: usize) -> Self {
        Self {
            services: vec![Service::default(); num_services],
            table_state: TableState::default().with_selected(None),
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
            Environment::Preproduction => service.preproduction = commit,
            Environment::Production => service.production = commit,
            _ => {}
        }
    }

    pub fn get_selected_service_idx(&self) -> Option<usize> {
        self.table_state.selected()
    }

    pub(crate) fn has_link(&self) -> bool {
        match self.table_state.selected() {
            Some(service_idx) => {
                let service = &self.services[service_idx];
                service.commit_ref_status() == CommitRefStatus::StagingPreprodMatch
            }
            None => false,
        }
    }

    pub(crate) fn get_link(&self, repo_url: &str) -> Option<String> {
        let service_idx = self.table_state.selected()?;
        let service = &self.services[service_idx];

        if let Some(prod_ref) = service.production.get_ref()
            && let Some(preprod_ref) = service.preproduction.get_ref()
        {
            Some(format!(
                "{}/compare/{}...{}",
                repo_url, prod_ref, preprod_ref,
            ))
        } else {
            None
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Commit {
    Empty,
    Fetching,
    Ok(String),
    Error(String),
}

impl Commit {
    pub fn get_ref(&self) -> Option<&str> {
        match self {
            Commit::Ok(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_error(&self) -> Option<&str> {
        match self {
            Commit::Error(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn short_value(&self) -> Option<String> {
        let n = 6;
        match self {
            Commit::Ok(s) if s.len() >= n * 2 => {
                Some(format!("{}...{}", &s[..n], &s[s.len() - n..]))
            }
            Commit::Ok(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn is_errored(&self) -> bool {
        matches!(self, Commit::Error(_))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Service {
    pub staging: Commit,
    pub preproduction: Commit,
    pub production: Commit,
}

#[derive(PartialEq, Debug)]
pub enum CommitRefStatus {
    NothingMatches,
    AllMatches,
    StagingPreprodMatch,
    PreprodProdMatch,
    CommitMissing,
}

impl Default for Service{
    fn default() -> Self {
        Self {
            staging: Commit::Empty,
            preproduction: Commit::Empty,
            production: Commit::Empty,
        }
    }
}

impl Service {
    pub fn commit_ref_status(&self) -> CommitRefStatus {
        if self.production.is_errored()
            || self.preproduction.is_errored()
            || self.staging.is_errored()
        {
            return CommitRefStatus::CommitMissing;
        }

        let preprod_prod_match = self.production.get_ref() == self.preproduction.get_ref();
        let staging_preprod_match = self.preproduction.get_ref() == self.staging.get_ref();

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

#[cfg(test)]
mod tests {
    use crate::environment::Environment;
    use crate::state::service_status::{Commit, CommitRefStatus, Service, ServiceStatus};
    use test_case::test_case;

    #[test]
    fn set_commit_fetching_sets_commit_as_fetching() {
        let mut service_status = ServiceStatus::new(2);
        service_status.set_commit_fetching(1, &Environment::Staging);
        assert_eq!(service_status.services[1].staging, Commit::Fetching);
    }

    #[test]
    fn set_commit_ok_sets_commit_as_ok() {
        let mut service_status = ServiceStatus::new(2);
        service_status.set_commit_ok(1, &Environment::Preproduction, String::from("commit_ref"));
        assert_eq!(
            service_status.services[1].preproduction,
            Commit::Ok(String::from("commit_ref"))
        );
    }

    #[test]
    fn set_commit_error_sets_commit_as_error() {
        let mut service_status = ServiceStatus::new(2);
        service_status.set_commit_error(1, &Environment::Production, String::from("error"));
        assert_eq!(
            service_status.services[1].production,
            Commit::Error(String::from("error"))
        );
    }

    #[test]
    fn update_commit_ignores_local_env() {
        let mut service_status = ServiceStatus::new(2);
        let expected = service_status.services[1].clone();
        service_status.update_commit(1, &Environment::Local, Commit::Fetching);
        assert_eq!(service_status.services[1], expected)
    }

    #[test]
    fn get_selected_service_idx_returns_selected() {
        let mut service_status = ServiceStatus::new(3);
        assert_eq!(service_status.get_selected_service_idx(), None);

        service_status.table_state.select(Some(2));
        assert_eq!(service_status.get_selected_service_idx(), Some(2));
    }

    #[test]
    fn has_link_returns_true_when_staging_preprod_match() {
        let mut service_status = ServiceStatus::new(2);
        let commit_ref = String::from("commit");
        service_status.services[1].staging = Commit::Ok(commit_ref.clone());
        service_status.services[1].preproduction = Commit::Ok(commit_ref);
        service_status.table_state.select(Some(1));

        assert!(service_status.has_link());
    }

    #[test]
    fn has_link_returns_false_when_staging_preprod_do_not_match() {
        let mut service_status = ServiceStatus::new(2);
        service_status.services[1].staging = Commit::Ok(String::from("staging"));
        service_status.services[1].preproduction = Commit::Ok(String::from("preproduction"));
        service_status.table_state.select(Some(1));

        assert!(!service_status.has_link());
    }

    #[test]
    fn get_link_returns_url_string() {
        let mut service_status = ServiceStatus::new(2);
        service_status.table_state.select(Some(1));

        service_status.services[1].preproduction = Commit::Ok(String::from("preprod"));
        service_status.services[1].production = Commit::Ok(String::from("prod"));

        let actual = service_status.get_link("https://github.com/myrepo");

        assert_eq!(
            actual.unwrap(),
            String::from("https://github.com/myrepo/compare/prod...preprod")
        );
    }

    #[test_case(None, Commit::Ok(String::from("preprod")), Commit::Ok(String::from("prod")); "nothing selected")]
    #[test_case(Some(1), Commit::Empty, Commit::Ok(String::from("rod")); "no preprod commit")]
    #[test_case(Some(1), Commit::Ok(String::from("preprod")), Commit::Empty; "no prod commit")]
    fn get_link_returns_none_when_required_values_are_not_set(
        selected: Option<usize>,
        preprod_commit: Commit,
        prod_commit: Commit,
    ) {
        let mut service_status = ServiceStatus::new(2);
        service_status.table_state.select(selected);

        service_status.services[1].preproduction = preprod_commit;
        service_status.services[1].production = prod_commit;

        assert_eq!(None, service_status.get_link("repo_url"));
    }

    #[test_case(Commit::Ok(String::from("commit")), Some("commit"); "Returns value from Ok Commit")]
    #[test_case(Commit::Fetching, None; "Returns NONE when no Commit::Ok")]
    fn commit_get_ref_returns_expected_value(commit: Commit, expected: Option<&str>) {
        assert_eq!(commit.get_ref(), expected);
    }

    #[test_case(Commit::Ok(String::from("commit")), None; "Returns NONE when no Commit::Ok")]
    #[test_case(Commit::Error(String::from("No good")), Some("No good"); "Returns value from Error Commit")]
    fn commit_get_error_returns_expected_value(commit: Commit, expected: Option<&str>) {
        assert_eq!(commit.get_error(), expected);
    }

    #[test_case(
        Commit::Ok(String::from("e5a18108ac6bc2c2a77ac3bda09ba1752c0087a5")),
        Some(String::from("e5a181...0087a5"));
        "Returns shortened commit ref"
    )]
    #[test_case(
        Commit::Ok(String::from("e5a18108ac6")),
        Some(String::from("e5a18108ac6"));
        "Returns full commit ref when less than 12 characters. "
    )]
    #[test_case(Commit::Fetching, None; "Returns NONE when Commit is no Ok")]
    fn commit_short_value_returns_expected_value(commit: Commit, expected: Option<String>) {
        assert_eq!(commit.short_value(), expected);
    }

    #[test_case(Commit::Empty, false; "Is errored returns false")]
    #[test_case(Commit::Error(String::from("error")), true; "Is errored returns true")]
    fn commit_is_errored_returns_expected_bool(commit: Commit, expected: bool) {
        assert_eq!(commit.is_errored(), expected);
    }

    #[test_case(
        Commit::Error(String::from("error")),
        Commit::Empty,
        Commit::Empty,
        CommitRefStatus::CommitMissing;
        "Commit marked as missing when staging is an error"
    )]
    #[test_case(
        Commit::Empty,
        Commit::Error(String::from("error")),
        Commit::Empty,
        CommitRefStatus::CommitMissing;
        "Commit marked as missing when preprod is an error"
    )]
    #[test_case(
        Commit::Empty,
        Commit::Empty,
        Commit::Error(String::from("error")),
        CommitRefStatus::CommitMissing;
        "Commit marked as missing when prod is an error"
    )]
    #[test_case(
        Commit::Ok(String::from("commit")),
        Commit::Ok(String::from("commit")),
        Commit::Ok(String::from("not_on_prod")),
        CommitRefStatus::StagingPreprodMatch;
        "Commit marked as StagingPreprodMatch when they do but don't match prod"
    )]
    #[test_case(
        Commit::Ok(String::from("commit")),
        Commit::Ok(String::from("commit")),
        Commit::Ok(String::from("commit")),
        CommitRefStatus::AllMatches;
        "Commit marked as AllMatches when all commit references match"
    )]
    #[test_case(
        Commit::Ok(String::from("incoming")),
        Commit::Ok(String::from("commit")),
        Commit::Ok(String::from("commit")),
        CommitRefStatus::PreprodProdMatch;
        "Commit marked as PreprodProdMatch when they do but doesn't match staging"
    )]
    #[test_case(
        Commit::Ok(String::from("staging")),
        Commit::Ok(String::from("preprod")),
        Commit::Ok(String::from("prod")),
        CommitRefStatus::NothingMatches;
        "Commit marked as NothingMatches when nothing matches"
    )]
    fn service_commit_ref_status(
        staging_commit: Commit,
        preprod_commit: Commit,
        prod_commit: Commit,
        expected: CommitRefStatus,
    ) {
        let mut service = Service::default();
        service.staging = staging_commit;
        service.preproduction = preprod_commit;
        service.production = prod_commit;

        assert_eq!(service.commit_ref_status(), expected);
    }
}
