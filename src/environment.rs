use serde::Deserialize;
use strum_macros::EnumCount;
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, EnumCount, Debug)]
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

#[cfg(test)]
mod tests {
    use crate::environment::Environment;

    #[test]
    fn as_str() {
        assert_eq!(Environment::Local.as_str(), "Local");
        assert_eq!(Environment::Staging.as_str(), "Staging");
        assert_eq!(Environment::Preproduction.as_str(), "Preproduction");
        assert_eq!(Environment::Production.as_str(), "Production");
    }
}
