use std::collections::HashMap;
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub audience: String,
    pub local: Option<Credentials>,
    pub staging: Option<Credentials>,
    pub preproduction: Option<Credentials>,
    pub production: Option<Credentials>
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String
}