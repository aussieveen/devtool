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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::jira::adf::inlinenodes::text::Text;
    use crate::client::jira::adf::nodes::InlineNode;

    #[test]
    fn test_empty_content() {
        let para = Paragraph { content: None };
        assert_eq!(para.to_markdown(), "");
    }

    #[test]
    fn test_with_text() {
        let para = Paragraph {
            content: Some(vec![InlineNode::Text(Text {
                text: "Hello world".to_string(),
                marks: None,
            })]),
        };
        assert_eq!(para.to_markdown(), "Hello world");
    }

    #[test]
    fn test_multiple_inline_nodes() {
        let para = Paragraph {
            content: Some(vec![
                InlineNode::Text(Text { text: "Hello".to_string(), marks: None }),
                InlineNode::HardBreak,
                InlineNode::Text(Text { text: "world".to_string(), marks: None }),
            ]),
        };
        assert_eq!(para.to_markdown(), "Hello  \nworld");
    }
}
