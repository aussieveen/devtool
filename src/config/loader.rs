use crate::config::model::Config;
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
    fn from_path(file_path: PathBuf) -> ConfigLoader {
        ConfigLoader { file_path }
    }

    pub fn read_config(self) -> Config {
        let config = fs::read_to_string(&self.file_path).expect("Unable to read content to string");
        serde_yaml::from_str(config.as_str()).expect("File does not match expected format")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{Auth0Config, ServiceStatus};
    use crate::environment::Environment;
    use tempfile::TempDir;

    #[test]
    fn service_status_get_from_env() {
        let status = ServiceStatus {
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
            "http://production.test.com"
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

    #[test]
    fn read_config_returns_config_successfully() {
        let yaml = "servicestatus:
  - name: My Api
    staging: https://myapi.staging.com/
    preproduction: https://myapi.preprod.com/
    production: https://myapi.prod.com/
    repo: https://github.com/myapi/
tokengenerator:
  auth0:
    # url structure https://m2m-auth0-url.com/oauth/token
    local: local_auth0
    staging: staging_auth0
    preproduction: preproduction_auth0
    production: production_auth0
  services:
    - name: my-user-service
      audience: user
      credentials:
        - env: Local
          client_id: abc
          client_secret: 123
        - env: Staging
          client_id: 123
          client_secret: 456
        - env: Preproduction
          client_id: 456
          client_secret: def
        - env: Production
          client_id: def
          client_secret: 789
jira:
    url: url
    email: email
    token: token";

        let dir = TempDir::new().unwrap();
        let file_path = temp_loader_path(&dir);
        fs::write(&file_path, yaml).expect("Unable to write temp config file");

        let config_loader = ConfigLoader::from_path(file_path);

        let config = config_loader.read_config();
        let servicestatus = &config.servicestatus[0];
        assert_eq!(servicestatus.name, "My Api");
        assert_eq!(servicestatus.staging, "https://myapi.staging.com/");
        assert_eq!(servicestatus.preproduction, "https://myapi.preprod.com/");
        assert_eq!(servicestatus.production, "https://myapi.prod.com/");
        assert_eq!(servicestatus.repo, "https://github.com/myapi/");

        let tokengen = &config.tokengenerator;
        assert_eq!(tokengen.auth0.local, "local_auth0");
        assert_eq!(tokengen.auth0.staging, "staging_auth0");
        assert_eq!(tokengen.auth0.preproduction, "preproduction_auth0");
        assert_eq!(tokengen.auth0.staging, "staging_auth0");

        assert_eq!(tokengen.services[0].name, "my-user-service");
        assert_eq!(tokengen.services[0].audience, "user");
        assert_eq!(tokengen.services[0].credentials[0].env, Environment::Local);
        assert_eq!(tokengen.services[0].credentials[0].client_id, "abc");
        assert_eq!(tokengen.services[0].credentials[0].client_secret, "123");
        assert_eq!(
            tokengen.services[0].credentials[1].env,
            Environment::Staging
        );
        assert_eq!(tokengen.services[0].credentials[1].client_id, "123");
        assert_eq!(tokengen.services[0].credentials[1].client_secret, "456");
        assert_eq!(
            tokengen.services[0].credentials[2].env,
            Environment::Preproduction
        );
        assert_eq!(tokengen.services[0].credentials[2].client_id, "456");
        assert_eq!(tokengen.services[0].credentials[2].client_secret, "def");
        assert_eq!(
            tokengen.services[0].credentials[3].env,
            Environment::Production
        );
        assert_eq!(tokengen.services[0].credentials[3].client_id, "def");
        assert_eq!(tokengen.services[0].credentials[3].client_secret, "789");

        let jira = &config.jira.unwrap();
        assert_eq!(jira.token, "token");
        assert_eq!(jira.email, "email");
    }

    #[test]
    #[should_panic(expected = "Unable to read content to string")]
    fn read_config_panics_when_unable_to_read_file_content() {
        let dir = TempDir::new().unwrap();
        let file_path = temp_loader_path(&dir);
        let config_loader = ConfigLoader::from_path(file_path);
        config_loader.read_config();
    }

    #[test]
    #[should_panic(expected = "File does not match expected format")]
    fn read_config_panics_when_unable_to_decode_yaml() {
        let dir = TempDir::new().unwrap();
        let file_path = temp_loader_path(&dir);
        let json = "{}";
        fs::write(&file_path, json).expect("Unable to write to temp file");
        let config_loader = ConfigLoader::from_path(file_path);
        config_loader.read_config();
    }

    fn temp_loader_path(dir: &TempDir) -> PathBuf {
        dir.path().join("config.yaml")
    }
}
