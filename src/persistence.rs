use crate::state::jira::Ticket;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

pub trait FilePersistence {
    const FOLDER: &str;
    const PERSISTENCE_FILE: &str;
    fn write_persistence(persistence: Persistence) -> Result<Persistence, Box<dyn Error>>;
    fn read_persistence() -> Result<Persistence, Box<dyn Error>>;
}

pub trait JiraPersistence {
    fn read_jira_persistence() -> Jira;
    fn write_jira_tickets(tickets: &[Ticket]) -> Result<Jira, Box<dyn Error>>;
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Persistence {
    pub jira: Jira,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Jira {
    pub tickets: Vec<Ticket>,
}

impl Jira {
    pub fn new() -> Jira {
        Jira {
            tickets: Vec::new(),
        }
    }
}

pub struct JiraFile {}

impl JiraPersistence for JiraFile {
    fn read_jira_persistence() -> Jira {
        let persistence = File::read_persistence();

        let persistence_state = match persistence {
            Ok(state) => state,
            Err(e) => panic!("Unexpected read error {}", e),
        };

        persistence_state.jira
    }

    fn write_jira_tickets(tickets: &[Ticket]) -> Result<Jira, Box<dyn Error>> {
        let mut persistence: Persistence = File::read_persistence()?;
        persistence.jira.tickets = tickets.to_owned();
        Ok(File::write_persistence(persistence)?.jira)
    }
}

pub struct File {}

impl FilePersistence for File {
    const FOLDER: &str = ".devtool";
    const PERSISTENCE_FILE: &str = "persistence.yaml";

    fn write_persistence(persistence: Persistence) -> Result<Persistence, Box<dyn Error>> {
        let home_dir = dirs::home_dir().expect("Could not find home directory");

        // Append your file path
        let file_path: PathBuf = home_dir.join(Self::FOLDER).join(Self::PERSISTENCE_FILE);

        let yaml_string = serde_yaml::to_string(&persistence)?;

        fs::write(file_path, yaml_string)?;

        Ok(persistence)
    }

    fn read_persistence() -> Result<Persistence, Box<dyn Error>> {
        let home_dir = dirs::home_dir().expect("Could not find home directory");

        let file_path: PathBuf = home_dir.join(Self::FOLDER).join(Self::PERSISTENCE_FILE);

        let persistence_yaml = match fs::read_to_string(&file_path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let default = Persistence { jira: Jira::new() };
                Self::write_persistence(default.clone()).expect("Failed to write persistence");

                serde_yaml::to_string(&default)?
            }
            Err(e) => return Err(Box::new(e)),
        };

        // Parse YAML into Persistence struct
        let parsed: Persistence = serde_yaml::from_str(&persistence_yaml)?;
        Ok(parsed)
    }
}
