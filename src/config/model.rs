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
    #[serde(alias = "audience", deserialize_with = "deserialize_audiences")]
    pub audiences: Vec<String>,
    pub credentials: Vec<Credentials>,
}

/// Accepts either the legacy single `audience: String` field or the new
/// `audiences: Vec<String>` list, so existing config files keep loading.
fn deserialize_audiences<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }

    match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(audience) => Ok(vec![audience]),
        OneOrMany::Many(audiences) => Ok(audiences),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_config_migrates_legacy_singular_audience_field() {
        let yaml = "name: svc\naudience: im-content-resolution-api\ncredentials: []";
        let service: ServiceConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            service.audiences,
            vec!["im-content-resolution-api".to_string()]
        );
    }

    #[test]
    fn service_config_reads_new_audiences_list_field() {
        let yaml = "name: svc\naudiences:\n  - one\n  - two\ncredentials: []";
        let service: ServiceConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            service.audiences,
            vec!["one".to_string(), "two".to_string()]
        );
    }

    #[test]
    fn service_config_serializes_using_new_audiences_key() {
        let service = ServiceConfig {
            name: "svc".to_string(),
            audiences: vec!["one".to_string(), "two".to_string()],
            credentials: vec![],
        };
        let yaml = serde_yaml::to_string(&service).unwrap();
        assert!(yaml.contains("audiences:"));
        assert!(!yaml.contains("audience:"));
    }
}
