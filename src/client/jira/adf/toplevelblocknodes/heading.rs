use crate::client::jira::adf::traits::ToMarkdown;
use crate::client::jira::adf::inlinenodes::text::Text;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Heading {
    pub attrs: Attributes,
    pub content: Vec<Text>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub level: usize
}

impl ToMarkdown for Heading {
    fn to_markdown(&self) -> String {
        let level = self.attrs.level;
        let mut md = "#".repeat(level);
        md.push(' ');
        for text in &self.content {
            md.push_str(text.to_markdown().as_str());
        }
        md
    }
}
