use std::sync::Arc;
use crate::client::healthcheck::healthcheck_client;
use crate::config::model::ServiceStatusConfig;
use crate::environment::Environment;
use crate::error::model::ClientError;
use crate::event::events::ServiceStatusEvent::{GetCommitRefErrored, GetCommitRefOk};
use crate::event::sender::EventSender;
use reqwest::Client;

pub trait HealthcheckApi: Send + Sync {
    fn commit_ref(
        &self,
        service_idx: usize,
        env: Environment,
        config: Arc<[ServiceStatusConfig]>,
        sender: EventSender,
    );
}

pub struct ImmediateHealthcheckApi {
    client: Client,
}

impl ImmediateHealthcheckApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for ImmediateHealthcheckApi {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthcheckApi for ImmediateHealthcheckApi {
    fn commit_ref(
        &self,
        service_idx: usize,
        env: Environment,
        config: Arc<[ServiceStatusConfig]>,
        sender: EventSender,
    ) {
        let client = self.client.clone();
        tokio::spawn(async move {
            match commit_ref(client, service_idx, &env, config).await {
                Ok(commit) => {
                    sender.send_service_status_event(GetCommitRefOk(commit, service_idx, env));
                }
                Err(err) => {
                    sender.send_service_status_event(GetCommitRefErrored(
                        err.to_string(),
                        service_idx,
                        env,
                    ));
                }
            }
        });
    }
}

async fn commit_ref(
    client: Client,
    service_idx: usize,
    env: &Environment,
    config: Arc<[ServiceStatusConfig]>,
) -> Result<String, ClientError> {
    let healthcheck_response =
        healthcheck_client::get(client, config[service_idx].config_for_env(env)).await?;

    Ok(parse_version(healthcheck_response.version))
}

fn parse_version(version: String) -> String {
    match version.split_once('_') {
        Some((prefix, _)) => prefix.to_string(),
        None => version,
    }
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
