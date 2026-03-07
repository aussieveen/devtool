use serde::{Deserialize, Serialize};
use crate::client::jira::adf::toplevelblocknodes::bullet_list::BulletList;
use crate::client::jira::adf::toplevelblocknodes::codeblock::CodeBlock;
use crate::client::jira::adf::toplevelblocknodes::ordered_list::OrderedList;
use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;
use crate::client::jira::adf::traits::ToMarkdown;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockQuote {
    pub content: Vec<Content>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag="type")]
pub enum Content{
    CodeBlock(CodeBlock),
    BulletList(BulletList),
    OrderedList(OrderedList),
    Paragraph(Paragraph)
}

impl ToMarkdown for Content {
    fn to_markdown(&self) -> String {
        match self {
            Content::CodeBlock(c) => c.to_markdown(),
            Content::BulletList(b) => b.to_markdown(),
            Content::OrderedList(o) => o.to_markdown(),
            Content::Paragraph(p) => p.to_markdown(),
        }
    }
}

impl ToMarkdown for BlockQuote {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        for node in &self.content {
            md.push_str(node.to_markdown().as_str());
        }
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
    fn test_empty() {
        let bq = BlockQuote { content: vec![] };
        assert_eq!(bq.to_markdown(), "");
    }

    #[test]
    fn test_with_paragraph() {
        let bq = BlockQuote {
            content: vec![Content::Paragraph(Paragraph {
                content: Some(vec![InlineNode::Text(Text {
                    text: "Quoted text".to_string(),
                    marks: None,
                })]),
            })],
        };
        assert_eq!(bq.to_markdown(), "Quoted text");
    }

    #[test]
    fn test_multiple_nodes() {
        let bq = BlockQuote {
            content: vec![
                Content::Paragraph(Paragraph {
                    content: Some(vec![InlineNode::Text(Text {
                        text: "First".to_string(),
                        marks: None,
                    })]),
                }),
                Content::Paragraph(Paragraph {
                    content: Some(vec![InlineNode::Text(Text {
                        text: "Second".to_string(),
                        marks: None,
                    })]),
                }),
            ],
        };
        assert_eq!(bq.to_markdown(), "FirstSecond");
    }
}
