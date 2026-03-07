use crate::client::jira::adf::traits::ToMarkdown;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    pub attrs: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub text: String
}

impl ToMarkdown for Emoji {
    fn to_markdown(&self) -> String {
        self.attrs.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_markdown() {
        let emoji = Emoji { attrs: Attributes { text: "😀".to_string() } };
        assert_eq!(emoji.to_markdown(), "😀");
    }
}
