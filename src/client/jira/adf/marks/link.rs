use serde::{Deserialize, Serialize};
use crate::client::jira::adf::traits::Apply;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub attrs: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub href: String
}

impl Apply for Link {
    fn apply(&self, string: &str) -> String {
        format!("{}({})", string, self.attrs.href)
    }
}
