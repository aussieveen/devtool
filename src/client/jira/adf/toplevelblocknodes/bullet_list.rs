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
    fn test_single_item() {
        let list = BulletList { content: vec![text_item("First")] };
        assert_eq!(list.to_markdown(), "- First\n");
    }

    #[test]
    fn test_multiple_items() {
        let list = BulletList {
            content: vec![text_item("First"), text_item("Second"), text_item("Third")],
        };
        assert_eq!(list.to_markdown(), "- First\n- Second\n- Third\n");
    }
}
