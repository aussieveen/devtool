use std::collections::HashMap;
use serde::Deserialize;
use crate::environment::Environment;

#[derive(Debug, Deserialize)]
pub(crate) struct Config{
    pub diffchecker: Vec<DiffChecker>,
    pub tokengenerator: TokenGenerator
}

#[derive(Debug, Deserialize)]
pub(crate) struct DiffChecker {
    pub name: String,
    pub preprod: String,
    pub prod: String,
    pub repo: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenGenerator {
    pub auth0: Auth0Config,
    pub services: Vec<ServiceConfig>
}

#[derive(Debug, Deserialize)]
pub struct Auth0Config {
    pub local: String,
    pub staging: String,
    pub preproduction: String,
    pub production: String,
}

impl Auth0Config {
    pub fn get_from_env(&self, env: &Environment) -> &String
    {
        match env {
            Environment::Local => &self.local,
            Environment::Staging => &self.staging,
            Environment::Preproduction => &self.preproduction,
            Environment::Production => &self.production
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub audience: String,
    pub credentials: Vec<Credentials>
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub env: Environment,
    pub client_id: String,
    pub client_secret: String
}