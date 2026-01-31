use crate::environment::Environment;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub servicestatus: Vec<ServiceStatus>,
    pub tokengenerator: TokenGenerator,
    pub jira: Option<JiraConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServiceStatus {
    pub name: String,
    pub staging: String,
    pub preprod: String,
    pub prod: String,
    pub repo: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenGenerator {
    pub auth0: Auth0Config,
    pub services: Vec<ServiceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Auth0Config {
    pub local: String,
    pub staging: String,
    pub preproduction: String,
    pub production: String,
}

impl Auth0Config {
    pub fn get_from_env(&self, env: &Environment) -> &String {
        match env {
            Environment::Local => &self.local,
            Environment::Staging => &self.staging,
            Environment::Preproduction => &self.preproduction,
            Environment::Production => &self.production,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub audience: String,
    pub credentials: Vec<Credentials>,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub env: Environment,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraConfig {
    pub email: String,
    pub token: String,
}

const FOLDER: &str = ".devtool";
const CONFIG_FILE: &str = "config.yaml";

pub fn read_config() -> Config {
    let home_dir = dirs::home_dir().expect("Could not find home directory");

    // Append your file path
    let file_path: PathBuf = home_dir.join(FOLDER).join(CONFIG_FILE);

    // Read the file
    let config = fs::read_to_string(file_path).unwrap();

    serde_yaml::from_str(config.as_str()).unwrap()
}
