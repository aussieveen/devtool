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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::jira::adf::inlinenodes::text::Text;
    use crate::client::jira::adf::nodes::{InlineNode, TopLevelBlockNode};
    use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;

    fn text_paragraph(s: &str) -> TopLevelBlockNode {
        TopLevelBlockNode::Paragraph(Paragraph {
            content: Some(vec![InlineNode::Text(Text { text: s.to_string(), marks: None })]),
        })
    }

    #[test]
    fn test_empty() {
        let desc = Description { content: vec![] };
        assert_eq!(desc.to_markdown(), "");
    }

    #[test]
    fn test_single_paragraph() {
        let desc = Description { content: vec![text_paragraph("Hello")] };
        assert_eq!(desc.to_markdown(), "Hello\n\n");
    }

    #[test]
    fn test_multiple_paragraphs() {
        let desc = Description {
            content: vec![text_paragraph("First"), text_paragraph("Second")],
        };
        assert_eq!(desc.to_markdown(), "First\n\nSecond\n\n");
    }

    #[test]
    fn test_rule_separator() {
        let desc = Description {
            content: vec![
                text_paragraph("Before"),
                TopLevelBlockNode::Rule,
                text_paragraph("After"),
            ],
        };
        assert_eq!(desc.to_markdown(), "Before\n\n---\n\nAfter\n\n");
    }
}