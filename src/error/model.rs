#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub title: String,
    pub originating_event: String,
    pub tool: String,
    pub description: String,
}

/// Errors from HTTP client operations (requests, response parsing, API-level errors).
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("{0}")]
    Api(String),
}

/// Errors from reading or writing the persistence file.
#[derive(thiserror::Error, Debug)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

/// Errors from loading the config file at startup.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Config file has invalid format: {0}")]
    Parse(#[from] serde_yaml::Error),
}
