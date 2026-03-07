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
