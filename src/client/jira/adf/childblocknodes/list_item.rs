use crate::client::jira::adf::traits::ToMarkdown;
use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub content: Vec<Paragraph>,
}

impl ToMarkdown for ListItem {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        for node in &self.content {
            md.push_str(node.to_markdown().as_str());
        }
        md.push('\n');
        md
    }
}
