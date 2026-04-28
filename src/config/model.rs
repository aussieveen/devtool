use crate::environment::Environment;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct Config {
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

impl Config {
    pub fn normalize(mut self) -> Self {
        self.normalize_urls();
        self.enforce_feature_invariants();
        self
    }

    fn normalize_urls(&mut self){
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
    }

    fn enforce_feature_invariants(&mut self){
        if self.servicestatus.is_empty() {
            self.features.service_status = false;
        }
        if self.tokengenerator.services.is_empty() {
            self.features.token_generator = false;
        }
        if self.jira.is_none() {
            self.features.jira = false;
        }
    }

    fn strip_trailing_slash(s: &str) -> String {
        s.trim_end_matches('/').to_string()
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
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

#[derive(Deserialize, Serialize, Clone, PartialEq, Default)]
pub(crate) struct TokenGenerator {
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
    pub fn get_from_env(&self, env: &Environment) -> &str {
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
