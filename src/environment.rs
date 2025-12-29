use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
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
