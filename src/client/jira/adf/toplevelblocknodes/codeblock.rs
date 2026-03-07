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
            md.push_str("\n");
            for node in v {
                md.push_str(node.to_markdown().as_str());
            }
        }
        md.push_str("\n```");
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_language_and_content() {
        let block = CodeBlock {
            attrs: Attributes { language: "rust".to_string() },
            content: Some(vec![Text { text: "fn main() {}".to_string(), marks: None }]),
        };
        assert_eq!(block.to_markdown(), "```rust\nfn main() {}\n```");
    }

    #[test]
    fn test_no_content() {
        let block = CodeBlock {
            attrs: Attributes { language: "".to_string() },
            content: None,
        };
        assert_eq!(block.to_markdown(), "```\n```");
    }

    #[test]
    fn test_no_language() {
        let block = CodeBlock {
            attrs: Attributes { language: "".to_string() },
            content: Some(vec![Text { text: "x = 1".to_string(), marks: None }]),
        };
        assert_eq!(block.to_markdown(), "```\nx = 1\n```");
    }
}
