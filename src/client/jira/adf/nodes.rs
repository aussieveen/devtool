use crate::client::jira::adf::toplevelblocknodes::bullet_list::BulletList;
use crate::client::jira::adf::traits::ToMarkdown;
use crate::client::jira::adf::toplevelblocknodes::heading::Heading;
use crate::client::jira::adf::inlinenodes::inline_card::InlineCard;
use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;
use crate::client::jira::adf::inlinenodes::text::Text;
use serde::{Deserialize, Serialize};
use crate::client::jira::adf::inlinenodes::date::Date;
use crate::client::jira::adf::inlinenodes::emoji::Emoji;
use crate::client::jira::adf::inlinenodes::mention::Mention;
use crate::client::jira::adf::inlinenodes::status::Status;
use crate::client::jira::adf::toplevelblocknodes::blockquote::BlockQuote;
use crate::client::jira::adf::toplevelblocknodes::codeblock::CodeBlock;
use crate::client::jira::adf::toplevelblocknodes::ordered_list::OrderedList;
use crate::client::jira::adf::toplevelblocknodes::panel::Panel;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum TopLevelBlockNode {
    BlockQuote(BlockQuote),
    BulletList(BulletList),
    CodeBlock(CodeBlock),
    Heading(Heading),
    OrderedList(OrderedList),
    Panel(Panel),
    Paragraph(Paragraph),
    Rule,
}

impl ToMarkdown for TopLevelBlockNode {
    fn to_markdown(&self) -> String {
        let mut md = match self {
            TopLevelBlockNode::BlockQuote(block_quote) => block_quote.to_markdown(),
            TopLevelBlockNode::BulletList(bullet_list) => bullet_list.to_markdown(),
            TopLevelBlockNode::CodeBlock(code_block) => code_block.to_markdown(),
            TopLevelBlockNode::Heading(heading) => heading.to_markdown(),
            TopLevelBlockNode::OrderedList(ordered_list) => ordered_list.to_markdown(),
            TopLevelBlockNode::Panel(panel) => panel.to_markdown(),
            TopLevelBlockNode::Paragraph(paragraph) => paragraph.to_markdown(),
            TopLevelBlockNode::Rule => "---".to_string(),
        };
        md.push_str("\n\n");
        md
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum InlineNode {
    Date(Date),
    Emoji(Emoji),
    HardBreak,
    InlineCard(InlineCard),
    Mention(Mention),
    Status(Status),
    Text(Text),
}

impl ToMarkdown for InlineNode {
    fn to_markdown(&self) -> String {
        match self {
            InlineNode::Date(date) => date.to_markdown(),
            InlineNode::Emoji(emoji) => emoji.to_markdown(),
            InlineNode::HardBreak => "  \n".to_string(),
            InlineNode::InlineCard(card) => card.to_markdown(),
            InlineNode::Mention(mention) => mention.to_markdown(),
            InlineNode::Status(status) => status.to_markdown(),
            InlineNode::Text(text) => text.to_markdown(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::jira::adf::inlinenodes::text::Text;
    use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;

    #[test]
    fn test_rule_renders_hr() {
        assert_eq!(TopLevelBlockNode::Rule.to_markdown(), "---\n\n");
    }

    #[test]
    fn test_top_level_node_appends_double_newline() {
        let node = TopLevelBlockNode::Paragraph(Paragraph { content: None });
        assert_eq!(node.to_markdown(), "\n\n");
    }

    #[test]
    fn test_hard_break() {
        assert_eq!(InlineNode::HardBreak.to_markdown(), "  \n");
    }

    #[test]
    fn test_inline_text_delegates() {
        let node = InlineNode::Text(Text { text: "hello".to_string(), marks: None });
        assert_eq!(node.to_markdown(), "hello");
    }
}
