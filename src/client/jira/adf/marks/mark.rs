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
        if let Mark::Subsup(subsup) = self {
            let symbol = subsup.get_symbol();
            return format!("{symbol}{string}{symbol}");
        }

        let symbol = match self {
            Mark::Code => "`",
            Mark::Em => "_",
            Mark::Strike => "~~",
            Mark::Strong => "**",
            _ => ""
        };
        format!("{symbol}{string}{symbol}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::jira::adf::marks::link::{Link, Attributes as LinkAttrs};
    use crate::client::jira::adf::marks::subsup::{Subsup, Attributes as SubsupAttrs};

    #[test]
    fn test_code() {
        assert_eq!(Mark::Code.apply("text"), "`text`");
    }

    #[test]
    fn test_em() {
        assert_eq!(Mark::Em.apply("text"), "_text_");
    }

    #[test]
    fn test_strike() {
        assert_eq!(Mark::Strike.apply("text"), "~~text~~");
    }

    #[test]
    fn test_strong() {
        assert_eq!(Mark::Strong.apply("text"), "**text**");
    }

    #[test]
    fn test_link() {
        let mark = Mark::Link(Link { attrs: LinkAttrs { href: "https://example.com".to_string() } });
        assert_eq!(mark.apply("click here"), "click here(https://example.com)");
    }

    #[test]
    fn test_subsup_sub() {
        let mark = Mark::Subsup(Subsup { attrs: SubsupAttrs { r#type: "sub".to_string() } });
        assert_eq!(mark.apply("2"), "^2^");
    }

    #[test]
    fn test_subsup_sup() {
        let mark = Mark::Subsup(Subsup { attrs: SubsupAttrs { r#type: "sup".to_string() } });
        assert_eq!(mark.apply("2"), "~2~");
    }
}