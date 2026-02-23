use crate::environment::Environment;
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq)]
pub(crate) struct Config {
    pub servicestatus: Vec<ServiceStatus>,
    pub tokengenerator: TokenGenerator,
    pub jira: Option<JiraConfig>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub(crate) struct ServiceStatus {
    pub name: String,
    pub staging: String,
    pub preproduction: String,
    pub production: String,
    pub repo: String,
}

impl ServiceStatus {
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
