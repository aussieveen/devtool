use crate::client::jira::adf::traits::ToMarkdown;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub attrs: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub text: String
}

impl ToMarkdown for Status {
    fn to_markdown(&self) -> String {
        self.attrs.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_markdown() {
        let status = Status { attrs: Attributes { text: "In Progress".to_string() } };
        assert_eq!(status.to_markdown(), "In Progress");
    }
}
