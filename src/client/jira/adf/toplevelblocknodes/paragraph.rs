use crate::client::jira::adf::nodes::InlineNode;
use serde::{Deserialize, Serialize};
use crate::client::jira::adf::traits::ToMarkdown;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Paragraph {
    pub content: Option<Vec<InlineNode>>,
}

impl ToMarkdown for Paragraph {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        if let Some(v) = &self.content {
            for node in v {
                md.push_str(node.to_markdown().as_str());
            }
        }
        md
    }
}
