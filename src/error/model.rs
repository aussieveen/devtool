#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub title: String,
    pub originating_event: String,
    pub tool: String,
    pub description: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("{0}")]
    Api(String),
}

/// Errors from loading or reading the config file.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse config: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("Failed to read config: {0}")]
    Read(#[from] std::io::Error),
}

/// Errors from reading or writing the persistence file.
#[derive(thiserror::Error, Debug)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
