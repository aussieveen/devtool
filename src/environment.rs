use serde::Deserialize;
use strum_macros::EnumCount;
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, EnumCount)]
pub enum Environment {
    Local,
    Staging,
    Preproduction,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> String {
        let str = match self {
            Environment::Local => "Local",
            Environment::Staging => "Staging",
            Environment::Preproduction => "Preproduction",
            Environment::Production => "Production",
        };

        str.to_string()
    }
}
