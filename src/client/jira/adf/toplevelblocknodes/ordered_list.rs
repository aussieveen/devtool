use crate::client::jira::adf::traits::ToMarkdown;
use crate::client::jira::adf::childblocknodes::list_item::ListItem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderedList {
    pub attrs: Option<Attributes>,
    pub content: Vec<ListItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub order: i64
}

impl ToMarkdown for OrderedList {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        // "1. " or "3. " where 3 comes from attributes.order
        let prefix = format!("{}. ",match self.attrs {
            Some(attributes) => {
                attributes.order
            }
            None => {
                1
            }
        });
        for list_item in &self.content {
            md.push_str(prefix.as_str());
            md.push_str(list_item.to_markdown().as_str())
        }
        md
    }
}
