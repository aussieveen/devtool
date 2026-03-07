use crate::client::jira::adf::nodes::TopLevelBlockNode;
use serde::{Deserialize, Serialize};
use crate::client::jira::adf::traits::ToMarkdown;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]

pub struct Panel {
    pub content: Option<Vec<TopLevelBlockNode>>,
    pub attrs: Attributes
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub panel_type: PanelType
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PanelType{
    Error,
    Info,
    Note,
    Success,
    Warning
}

impl ToMarkdown for Panel {
    fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push(match self.attrs.panel_type{
            PanelType::Error => '✗',
            PanelType::Info => 'ℹ',
            PanelType::Note => '✎',
            PanelType::Success => '✓',
            PanelType::Warning => '⚠'
        });
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
    use crate::client::jira::adf::toplevelblocknodes::paragraph::Paragraph;

    fn panel_with_text(panel_type: PanelType, text: &str) -> Panel {
        Panel {
            attrs: Attributes { panel_type },
            content: Some(vec![TopLevelBlockNode::Paragraph(Paragraph {
                content: Some(vec![InlineNode::Text(Text { text: text.to_string(), marks: None })]),
            })]),
        }
    }

    #[test]
    fn test_info() {
        assert_eq!(panel_with_text(PanelType::Info, "message").to_markdown(), "ℹmessage\n\n");
    }

    #[test]
    fn test_warning() {
        assert_eq!(panel_with_text(PanelType::Warning, "caution").to_markdown(), "⚠caution\n\n");
    }

    #[test]
    fn test_error() {
        assert_eq!(panel_with_text(PanelType::Error, "failed").to_markdown(), "✗failed\n\n");
    }

    #[test]
    fn test_success() {
        assert_eq!(panel_with_text(PanelType::Success, "done").to_markdown(), "✓done\n\n");
    }

    #[test]
    fn test_note() {
        assert_eq!(panel_with_text(PanelType::Note, "read this").to_markdown(), "✎read this\n\n");
    }

    #[test]
    fn test_no_content() {
        let p = Panel { attrs: Attributes { panel_type: PanelType::Info }, content: None };
        assert_eq!(p.to_markdown(), "ℹ");
    }
}
