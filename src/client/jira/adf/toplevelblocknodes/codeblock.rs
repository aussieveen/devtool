use serde::{Deserialize, Serialize};
use crate::client::jira::adf::inlinenodes::text::Text;
use crate::client::jira::adf::traits::ToMarkdown;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeBlock {
    pub content: Option<Vec<Text>>,
    pub attrs: Attributes
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub language: String
}

impl ToMarkdown for CodeBlock {
    fn to_markdown(&self) -> String {
        let mut md = format!("```{}", self.attrs.language);
        if let Some(v) = &self.content {
            for node in v {
                md.push_str(node.to_markdown().as_str());
            }
        }
        md.push_str("\n```");
        md
    }
}
