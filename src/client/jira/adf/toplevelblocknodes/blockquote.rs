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

impl ToMarkdown for BlockQuote {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        for node in &self.content {
            md.push_str(node.to_markdown().as_str());
        }
        md
    }
}
