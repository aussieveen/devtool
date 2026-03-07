use crate::client::jira::adf::traits::{Apply, ToMarkdown};
use crate::client::jira::adf::marks::mark::Mark;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Text {
    pub text: String,
    pub marks: Option<Vec<Mark>>,
}

impl ToMarkdown for Text {
    fn to_markdown(&self) -> String {
        let mut md = Cow::Borrowed(self.text.as_str());
        if let Some(marks) = &self.marks {
            for mark in marks {
                md = Cow::Owned(mark.apply(&md));
            }
        }
        md.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::jira::adf::marks::mark::Mark;

    #[test]
    fn test_plain_text() {
        let text = Text { text: "hello".to_string(), marks: None };
        assert_eq!(text.to_markdown(), "hello");
    }

    #[test]
    fn test_no_marks() {
        let text = Text { text: "hello".to_string(), marks: Some(vec![]) };
        assert_eq!(text.to_markdown(), "hello");
    }

    #[test]
    fn test_bold() {
        let text = Text { text: "hello".to_string(), marks: Some(vec![Mark::Strong]) };
        assert_eq!(text.to_markdown(), "**hello**");
    }

    #[test]
    fn test_italic() {
        let text = Text { text: "hello".to_string(), marks: Some(vec![Mark::Em]) };
        assert_eq!(text.to_markdown(), "_hello_");
    }

    #[test]
    fn test_code() {
        let text = Text { text: "x".to_string(), marks: Some(vec![Mark::Code]) };
        assert_eq!(text.to_markdown(), "`x`");
    }

    #[test]
    fn test_strike() {
        let text = Text { text: "hello".to_string(), marks: Some(vec![Mark::Strike]) };
        assert_eq!(text.to_markdown(), "~~hello~~");
    }

    #[test]
    fn test_marks_applied_in_order() {
        // Strong applied first, then Em wraps the result
        let text = Text { text: "hello".to_string(), marks: Some(vec![Mark::Strong, Mark::Em]) };
        assert_eq!(text.to_markdown(), "_**hello**_");
    }
}
