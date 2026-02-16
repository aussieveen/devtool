use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Healthcheck {
    pub version: String,
}

#[cfg(test)]
mod tests {
    use crate::client::healthcheck::models::Healthcheck;

    #[test]
    fn deserialize_healthcheck_response() {
        let json = r#"{
            "version": "version_timestamp"
        }"#;
        let healthcheck: Healthcheck = serde_json::from_str(json).unwrap();
        assert_eq!(healthcheck.version, "version_timestamp");
    }
}
