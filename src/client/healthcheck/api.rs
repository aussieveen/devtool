use crate::client::healthcheck::healthcheck_client;
use crate::config::model::ServiceStatus;
use crate::environment::Environment;
use crate::error::model::ClientError;
use crate::events::event::AppEvent::{GetCommitRefErrored, GetCommitRefOk};
use crate::events::sender::EventSender;

pub trait HealthcheckApi {
    fn get_commit_ref(
        &self,
        service_idx: usize,
        env: Environment,
        config: Vec<ServiceStatus>,
        sender: EventSender,
    );
}

pub struct ImmediateHealthcheckApi {
    client: reqwest::Client,
}

impl ImmediateHealthcheckApi {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl HealthcheckApi for ImmediateHealthcheckApi {
    fn get_commit_ref(
        &self,
        service_idx: usize,
        env: Environment,
        config: Vec<ServiceStatus>,
        sender: EventSender,
    ) {
        let client = self.client.clone();
        tokio::spawn(async move {
            match get_commit_ref(&client, service_idx, &env, config).await {
                Ok(commit) => {
                    sender.send(GetCommitRefOk(commit, service_idx, env));
                }
                Err(err) => {
                    sender.send(GetCommitRefErrored(err.to_string(), service_idx, env));
                }
            }
        });
    }
}

async fn get_commit_ref(
    client: &reqwest::Client,
    service_idx: usize,
    env: &Environment,
    config: Vec<ServiceStatus>,
) -> Result<String, ClientError> {
    let healthcheck_response =
        healthcheck_client::get(client, config[service_idx].get_from_env(env)).await?;

    Ok(parse_version(healthcheck_response.version))
}

fn parse_version(version: String) -> String {
    version.split('_').next().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use crate::client::healthcheck::api::parse_version;
    use test_case::test_case;

    #[test_case("a_b".to_string(), "a".to_string(); "Version parsed with _")]
    #[test_case("ab".to_string(), "ab".to_string(); "Version without _")]
    fn api_parse_version(version: String, expected: String) {
        assert_eq!(parse_version(version), expected);
    }
}
