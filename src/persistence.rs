use std::error::Error;
use std::{fs};
use std::path::PathBuf;
use serde::Deserialize;
use crate::state::jira::Ticket;

#[derive(Debug, Deserialize)]
pub (crate) struct Persistence {
    pub jira: Option<Jira>
}

#[derive(Debug, Deserialize)]
pub (crate) struct Jira {
    pub tickets: Vec<Ticket>
}

const FOLDER: &str = ".devtool";
const PERSISTENCE_FILE: &str = "persistence.yaml";

pub fn read_jira_persistence() -> Option<Jira>
{
    let persistence = read_persistence();

    let persistence_state = match persistence {
        Ok(state) => state,
        Err(_) => return None,
    };

    persistence_state.jira
}

pub fn read_persistence() -> Result<Persistence, Box<dyn Error>>{
    let home_dir = dirs::home_dir().expect("Could not find home directory");

    // Append your file path
    let file_path: PathBuf = home_dir.join(FOLDER).join(PERSISTENCE_FILE);

    // Read the file
    let persistence = fs::read_to_string(file_path)?;

    // Parse to Persistence
    let parsed = serde_yaml::from_str(persistence.as_str())?;

    Ok(parsed)
}

