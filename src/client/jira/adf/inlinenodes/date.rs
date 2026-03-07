use chrono::{DateTime, Utc};
use crate::client::jira::adf::traits::ToMarkdown;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Date {
    pub attrs: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub timestamp: String
}

impl ToMarkdown for Date {
    fn to_markdown(&self) -> String {
        let timestamp: i64 = self.attrs.timestamp.parse().unwrap_or_default();
        match DateTime::from_timestamp_secs(timestamp) {
            Some(dt) => dt.to_rfc2822(),
            None => self.attrs.timestamp.clone(),
        }
    }
}
