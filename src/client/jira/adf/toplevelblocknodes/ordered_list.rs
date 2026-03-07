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
        let prefix = format!("{}. ",match self.attrs.clone() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::jira::adf::childblocknodes::list_item::ListItem;
    use crate::client::jira::adf::inlinenodes::text::Text;
    use crate::client::jira::adf::nodes::InlineNode;
    use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;

    fn text_item(s: &str) -> ListItem {
        ListItem {
            content: vec![Paragraph {
                content: Some(vec![InlineNode::Text(Text { text: s.to_string(), marks: None })]),
            }],
        }
    }

    #[test]
    fn test_default_start() {
        let list = OrderedList {
            attrs: None,
            content: vec![text_item("First"), text_item("Second"), text_item("Third")],
        };
        assert_eq!(list.to_markdown(), "1. First\n1. Second\n1. Third\n");
    }

    #[test]
    fn test_custom_start() {
        let list = OrderedList {
            attrs: Some(Attributes { order: 3 }),
            content: vec![text_item("First"), text_item("Second")],
        };
        assert_eq!(list.to_markdown(), "3. First\n3. Second\n");
    }

    #[test]
    fn test_single_item_defaults_to_one() {
        let list = OrderedList {
            attrs: None,
            content: vec![text_item("Only")],
        };
        assert_eq!(list.to_markdown(), "1. Only\n");
    }
}
