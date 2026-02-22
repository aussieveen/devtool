use std::fmt::{Display, Formatter};
use serde::Deserialize;
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Debug)]
pub enum Environment {
    Local,
    Staging,
    Preproduction,
    Production,
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Environment::Local => "Local",
            Environment::Staging => "Staging",
            Environment::Preproduction => "Preproduction",
            Environment::Production => "Production",
        };

        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod tests {
    use crate::environment::Environment;

    #[test]
    fn display_fmt() {
        assert_eq!(Environment::Local.to_string(), "Local");
        assert_eq!(Environment::Staging.to_string(), "Staging");
        assert_eq!(Environment::Preproduction.to_string(), "Preproduction");
        assert_eq!(Environment::Production.to_string(), "Production");
    }
}
