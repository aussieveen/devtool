use serde::{Deserialize, Serialize};
use crate::client::jira::adf::marks::link::Link;
use crate::client::jira::adf::marks::subsup::Subsup;
use crate::client::jira::adf::traits::Apply;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Mark {
    // Border, TextColor and Underline are also options which MD doesn't really support
    Code,
    Em,
    Link(Link),
    Strike,
    Strong,
    Subsup(Subsup)
}

impl Apply for Mark {
    fn apply(&self, string: &str) -> String {
        if let Mark::Link(link) = self { return link.apply(string); }

        let symbol = match self {
            Mark::Code => "`",
            Mark::Em => "_",
            Mark::Strike => "~~",
            Mark::Strong => "**",
            Mark::Subsup(subsup) => subsup.get_symbol().as_str(),
            _ => ""
        };
        format!("{symbol}{string}{symbol}")
    }
}