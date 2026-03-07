use crate::client::jira::adf::nodes::TopLevelBlockNode;
use serde::{Deserialize, Serialize};
use crate::client::jira::adf::traits::ToMarkdown;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Description {
    pub content: Vec<TopLevelBlockNode>,
}

impl ToMarkdown for Description {
    fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        for node in &self.content {
            markdown.push_str(node.to_markdown().as_str());
        }
        markdown
    }
}