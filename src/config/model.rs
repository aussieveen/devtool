use crate::environment::Environment;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Config {
    pub servicestatus: Vec<ServiceStatusConfig>,
    pub tokengenerator: TokenGenerator,
    pub jira: Option<JiraConfig>,
    #[serde(default)]
    pub features: Features,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Features {
    #[serde(default = "default_true")]
    pub service_status: bool,
    #[serde(default = "default_true")]
    pub token_generator: bool,
    #[serde(default = "default_true")]
    pub jira: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            service_status: true,
            token_generator: true,
            jira: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servicestatus: Vec::new(),
            tokengenerator: TokenGenerator::default(),
            jira: None,
            features: Features {
                service_status: false,
                token_generator: false,
                jira: false,
            },
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct ServiceStatusConfig {
    pub name: String,
    pub staging: String,
    pub preproduction: String,
    pub production: String,
    pub repo: String,
}

impl ServiceStatusConfig {
    pub fn config_for_env(&self, env: &Environment) -> &str {
        match env {
            Environment::Local => &self.staging,
            Environment::Staging => &self.staging,
            Environment::Preproduction => &self.preproduction,
            Environment::Production => &self.production,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct TokenGenerator {
    pub auth0: Auth0Config,
    pub services: Vec<ServiceConfig>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Auth0Config {
    pub local: String,
    pub staging: String,
    pub preproduction: String,
    pub production: String,
}

impl Auth0Config {
    pub fn config_for_env(&self, env: &Environment) -> &str {
        match env {
            Environment::Local => &self.local,
            Environment::Staging => &self.staging,
            Environment::Preproduction => &self.preproduction,
            Environment::Production => &self.production,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct ServiceConfig {
    pub name: String,
    pub audience: String,
    pub credentials: Vec<Credentials>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Credentials {
    pub env: Environment,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct JiraConfig {
    pub url: String,
    pub email: String,
    pub token: String,
}
