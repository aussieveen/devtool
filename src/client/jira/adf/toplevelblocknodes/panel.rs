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
