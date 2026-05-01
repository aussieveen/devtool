use crate::config::model::Config;
use crate::error::model::ConfigError;
use std::fs;
use std::path::PathBuf;

pub struct ConfigLoader {
    file_path: PathBuf,
}

impl ConfigLoader {
    pub fn new(folder: &str, config_file: &str) -> ConfigLoader {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        ConfigLoader {
            file_path: home_dir.join(folder).join(config_file),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_path(file_path: PathBuf) -> ConfigLoader {
        ConfigLoader { file_path }
    }

    pub fn read_or_create_config(&self) -> Result<Config, ConfigError> {
        match fs::read_to_string(&self.file_path) {
            Ok(content) => Ok(serde_yaml::from_str::<Config>(content.as_str())?.normalize()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let config = Config::default();
                if let Some(parent) = self.file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let yaml = serde_yaml::to_string(&config)?;
                fs::write(&self.file_path, yaml)?;
                Ok(config)
            }
            Err(e) => Err(ConfigError::Read(e)),
        }
    }

    pub fn write_config(&self, config: &Config) -> Result<(), ConfigError> {
        let yaml = serde_yaml::to_string(config)?;
        fs::write(&self.file_path, yaml)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{Auth0Config, ServiceStatusConfig};
    use crate::environment::Environment;
    use tempfile::TempDir;

    #[test]
    fn service_status_get_from_env() {
        let status = ServiceStatusConfig {
            name: "test_service_status".to_string(),
            staging: "http://staging.test.com".to_string(),
            preproduction: "http://preproduction.test.com".to_string(),
            production: "http://production.test.com".to_string(),
            repo: "http://repo.test.com".to_string(),
        };
        assert_eq!(
            status.get_from_env(&Environment::Staging),
            "http://staging.test.com"
        );
        assert_eq!(
            status.get_from_env(&Environment::Preproduction),
            "http://preproduction.test.com"
        );
        assert_eq!(
            status.get_from_env(&Environment::Production),
            "http://production.test.com"
        );
        assert_eq!(
            status.get_from_env(&Environment::Local),
            "http://staging.test.com"
        );
    }

    #[test]
    fn auth0_config_get_from_env() {
        let config = Auth0Config {
            local: "local".to_string(),
            staging: "staging".to_string(),
            preproduction: "preproduction".to_string(),
            production: "production".to_string(),
        };

        assert_eq!(config.get_from_env(&Environment::Local), "local");
        assert_eq!(config.get_from_env(&Environment::Staging), "staging");
        assert_eq!(
            config.get_from_env(&Environment::Preproduction),
            "preproduction"
        );
        assert_eq!(config.get_from_env(&Environment::Production), "production");
    }

    fn temp_loader_path(dir: &TempDir) -> PathBuf {
        dir.path().join("config.yaml")
    }

    #[test]
    fn read_or_create_config_creates_file_when_missing() {
        let dir = TempDir::new().unwrap();
        let file_path = temp_loader_path(&dir);
        assert!(!file_path.exists());

        let config_loader = ConfigLoader::from_path(file_path.clone());
        let config = config_loader.read_or_create_config().unwrap();

        // Returns default config with all features disabled
        assert!(config.servicestatus.is_empty());
        assert!(config.tokengenerator.services.is_empty());
        assert!(config.jira.is_none());
        assert!(!config.features.service_status);
        assert!(!config.features.token_generator);
        assert!(!config.features.jira);

        // File was written to disk
        assert!(file_path.exists());
    }

    #[test]
    fn read_or_create_config_reads_existing_config() {
        let yaml = "servicestatus:
  - name: My Api
    staging: https://myapi.staging.com/
    preproduction: https://myapi.preprod.com/
    production: https://myapi.prod.com/
    repo: https://github.com/myapi/
tokengenerator:
  auth0:
    local: local_auth0
    staging: staging_auth0
    preproduction: preproduction_auth0
    production: production_auth0
  services: []";

        let dir = TempDir::new().unwrap();
        let file_path = temp_loader_path(&dir);
        fs::write(&file_path, yaml).expect("Unable to write temp config file");

        let config_loader = ConfigLoader::from_path(file_path);
        let config = config_loader.read_or_create_config().unwrap();

        assert_eq!(config.servicestatus[0].name, "My Api");
        assert_eq!(config.tokengenerator.auth0.local, "local_auth0");
    }
}
