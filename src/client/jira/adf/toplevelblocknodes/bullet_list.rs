use crate::client::jira::adf::traits::ToMarkdown;
use crate::client::jira::adf::childblocknodes::list_item::ListItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BulletList {
    pub content: Vec<ListItem>,
}

impl ToMarkdown for BulletList {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        for list_item in &self.content {
            md.push_str("- ");
            md.push_str(list_item.to_markdown().as_str())
        }
        md
    }
}
