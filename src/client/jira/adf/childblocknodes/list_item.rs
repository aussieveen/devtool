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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::jira::adf::inlinenodes::text::Text;
    use crate::client::jira::adf::nodes::InlineNode;
    use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;

    #[test]
    fn test_single_paragraph() {
        let item = ListItem {
            content: vec![Paragraph {
                content: Some(vec![InlineNode::Text(Text {
                    text: "item text".to_string(),
                    marks: None,
                })]),
            }],
        };
        assert_eq!(item.to_markdown(), "item text\n");
    }

    #[test]
    fn test_empty_paragraph() {
        let item = ListItem { content: vec![Paragraph { content: None }] };
        assert_eq!(item.to_markdown(), "\n");
    }
}
