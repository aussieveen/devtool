use crate::client::healthcheck::healthcheck_client;
use crate::config::model::ServiceStatus;
use crate::environment::Environment;
use crate::events::event::AppEvent::{GetCommitRefErrored, GetCommitRefOk};
use crate::events::sender::EventSender;
use std::error::Error;

pub trait HealthcheckApi {
    fn get_commit_ref(
        &self,
        service_idx: usize,
        env: Environment,
        config: Vec<ServiceStatus>,
        sender: EventSender,
    );
}

pub struct ImmediateHealthcheckApi {}

impl HealthcheckApi for ImmediateHealthcheckApi {
    fn get_commit_ref(
        &self,
        service_idx: usize,
        env: Environment,
        config: Vec<ServiceStatus>,
        sender: EventSender,
    ) {
        tokio::spawn(async move {
            match get_commit_ref(service_idx, &env, config).await {
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
    service_idx: usize,
    env: &Environment,
    config: Vec<ServiceStatus>,
) -> Result<String, Box<dyn Error>> {
    let healthcheck_response = healthcheck_client::get(config[service_idx].get_from_env(env).to_string()).await?;

    Ok(healthcheck_response.version
        .split("_")
        .next()
        .unwrap()
        .to_string())
}
