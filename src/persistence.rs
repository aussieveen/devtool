use std::error::Error;
use std::{fs};
use std::io::ErrorKind;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::state::jira::Ticket;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub (crate) struct Persistence {
    pub jira: Jira
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub (crate) struct Jira {
    pub tickets: Vec<Ticket>
}

impl Jira {
    pub fn new() -> Jira {
        Jira{
            tickets: Vec::new()
        }
    }
}

const FOLDER: &str = ".devtool";
const PERSISTENCE_FILE: &str = "persistence.yaml";

pub fn read_jira_persistence() -> Jira
{
    let persistence = read_persistence();

    let persistence_state = match persistence {
        Ok(state) => state,
        Err(e) => panic!("Unexpected read error {}", e)
    };

    persistence_state.jira
}

pub fn write_jira_tickets(tickets: &Vec<Ticket>) -> Result<Jira, Box<dyn Error>>
{
    let mut persistence: Persistence = read_persistence()?;
    persistence.jira.tickets = tickets.clone();
    Ok(write_persistence(persistence)?.jira)
}

fn write_jira_persistence(jira_update: Jira) -> Result<Persistence, Box<dyn Error>>
{
    let mut persistence = read_persistence()?;
    persistence.jira = jira_update;
    write_persistence(persistence)
}

pub fn write_persistence(persistence: Persistence) -> Result<Persistence, Box<dyn Error>>
{
    let home_dir = dirs::home_dir().expect("Could not find home directory");

    // Append your file path
    let file_path: PathBuf = home_dir.join(FOLDER).join(PERSISTENCE_FILE);

    let yaml_string = serde_yaml::to_string(&persistence)?;

    fs::write(file_path, yaml_string)?;

    Ok(persistence)
}

pub fn read_persistence() -> Result<Persistence, Box<dyn Error>>{
    let home_dir = dirs::home_dir().expect("Could not find home directory");

    let file_path: PathBuf = home_dir.join(FOLDER).join(PERSISTENCE_FILE);

    let persistence_yaml = match fs::read_to_string(&file_path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let default = Persistence { jira: Jira::new() };
            match write_persistence(default.clone()) {
                Ok(_) => {}
                Err(_) => {}
            };

            let yaml = serde_yaml::to_string(&default)?;
            yaml
        }
        Err(e) => return Err(Box::new(e))
    };

    // Parse YAML into Persistence struct
    let parsed: Persistence = serde_yaml::from_str(&persistence_yaml)?;
    Ok(parsed)
}

