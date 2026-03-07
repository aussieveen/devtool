use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Subsup {
    pub attrs: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attributes {
    pub r#type: String
}

impl Subsup {
    pub(crate) fn get_symbol(&self) -> String {
        let symbol = match self.attrs.r#type.as_str() {
            "sub" => "^",
            "sup" => "~",
            _ => ""
        };
        symbol.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_symbol() {
        let subsup = Subsup { attrs: Attributes { r#type: "sub".to_string() } };
        assert_eq!(subsup.get_symbol(), "^");
    }

    #[test]
    fn test_sup_symbol() {
        let subsup = Subsup { attrs: Attributes { r#type: "sup".to_string() } };
        assert_eq!(subsup.get_symbol(), "~");
    }

    #[test]
    fn test_unknown_symbol() {
        let subsup = Subsup { attrs: Attributes { r#type: "other".to_string() } };
        assert_eq!(subsup.get_symbol(), "");
    }
}
