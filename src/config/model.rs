use crate::environment::Environment;
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq)]
pub(crate) struct Config {
    pub servicestatus: Vec<ServiceStatusConfig>,
    pub tokengenerator: TokenGenerator,
    pub jira: Option<JiraConfig>,
}

impl Config {
    pub fn normalize(mut self) -> Self {
        for service in &mut self.servicestatus {
            service.staging = Self::strip_trailing_slash(&service.staging);
            service.preproduction = Self::strip_trailing_slash(&service.preproduction);
            service.production = Self::strip_trailing_slash(&service.production);
            service.repo = Self::strip_trailing_slash(&service.repo);
        }

        self.tokengenerator.auth0.local =
            Self::strip_trailing_slash(&self.tokengenerator.auth0.local);
        self.tokengenerator.auth0.staging =
            Self::strip_trailing_slash(&self.tokengenerator.auth0.staging);
        self.tokengenerator.auth0.preproduction =
            Self::strip_trailing_slash(&self.tokengenerator.auth0.preproduction);
        self.tokengenerator.auth0.production =
            Self::strip_trailing_slash(&self.tokengenerator.auth0.production);

        if let Some(ref mut jira) = self.jira {
            jira.url = Self::strip_trailing_slash(&jira.url);
        }
        self
    }

    fn strip_trailing_slash(s: &str) -> String {
        s.trim_end_matches('/').to_string()
    }
}

#[derive(Deserialize, Clone, PartialEq)]
pub(crate) struct ServiceStatusConfig {
    pub name: String,
    pub staging: String,
    pub preproduction: String,
    pub production: String,
    pub repo: String,
}

impl ServiceStatusConfig {
    pub fn get_from_env(&self, env: &Environment) -> &str {
        match env {
            Environment::Local => &self.staging,
            Environment::Staging => &self.staging,
            Environment::Preproduction => &self.preproduction,
            Environment::Production => &self.production,
        }
    }
}

#[derive(Deserialize, Clone, PartialEq)]
pub(crate) struct TokenGenerator {
    pub auth0: Auth0Config,
    pub services: Vec<ServiceConfig>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Auth0Config {
    pub local: String,
    pub staging: String,
    pub preproduction: String,
    pub production: String,
}

impl Auth0Config {
    pub fn get_from_env(&self, env: &Environment) -> &str {
        match env {
            Environment::Local => &self.local,
            Environment::Staging => &self.staging,
            Environment::Preproduction => &self.preproduction,
            Environment::Production => &self.production,
        }
    }
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct ServiceConfig {
    pub name: String,
    pub audience: String,
    pub credentials: Vec<Credentials>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Credentials {
    pub env: Environment,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct JiraConfig {
    pub url: String,
    pub email: String,
    pub token: String,
}
