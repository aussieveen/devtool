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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_timestamp() {
        let date = Date { attrs: Attributes { timestamp: "0".to_string() } };
        assert_eq!(date.to_markdown(), "Thu, 1 Jan 1970 00:00:00 +0000");
    }

    #[test]
    fn test_out_of_range_timestamp_falls_back_to_raw() {
        let ts = i64::MAX.to_string();
        let date = Date { attrs: Attributes { timestamp: ts.clone() } };
        assert_eq!(date.to_markdown(), ts);
    }

    #[test]
    fn test_unparseable_string_uses_epoch() {
        // Non-numeric strings parse to 0 (unwrap_or_default), showing epoch date
        let date = Date { attrs: Attributes { timestamp: "not-a-number".to_string() } };
        assert_eq!(date.to_markdown(), "Thu, 1 Jan 1970 00:00:00 +0000");
    }
}
