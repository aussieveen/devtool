use crate::client::jira::adf::traits::ToMarkdown;
use crate::client::jira::adf::inlinenodes::text::Text;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Heading {
    pub attrs: Attributes,
    pub content: Vec<Text>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub level: usize
}

impl ToMarkdown for Heading {
    fn to_markdown(&self) -> String {
        let level = self.attrs.level;
        let mut md = "#".repeat(level);
        md.push(' ');
        for text in &self.content {
            md.push_str(text.to_markdown().as_str());
        }
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_heading(level: usize, text: &str) -> Heading {
        Heading {
            attrs: Attributes { level },
            content: vec![Text { text: text.to_string(), marks: None }],
        }
    }

    #[test]
    fn test_h1() {
        assert_eq!(make_heading(1, "Title").to_markdown(), "# Title");
    }

    #[test]
    fn test_h2() {
        assert_eq!(make_heading(2, "Section").to_markdown(), "## Section");
    }

    #[test]
    fn test_h6() {
        assert_eq!(make_heading(6, "Deep").to_markdown(), "###### Deep");
    }

    #[test]
    fn test_multiple_text_nodes() {
        let heading = Heading {
            attrs: Attributes { level: 1 },
            content: vec![
                Text { text: "Hello".to_string(), marks: None },
                Text { text: " World".to_string(), marks: None },
            ],
        };
        assert_eq!(heading.to_markdown(), "# Hello World");
    }
}
