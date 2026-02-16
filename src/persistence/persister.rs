use crate::persistence::model::{Jira, Persistence};
use crate::state::jira::Ticket;
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

impl Jira {
    pub fn new() -> Jira {
        Jira {
            tickets: Vec::new(),
        }
    }
}

pub struct JiraFile {
    file: PersistenceFile,
}

impl JiraFile {
    pub fn new() -> JiraFile {
        JiraFile {
            file: PersistenceFile::new(),
        }
    }

    #[cfg(test)]
    fn new_from_path(file_path: PathBuf) -> JiraFile {
        JiraFile {
            file: PersistenceFile::new_from_path(file_path),
        }
    }

    pub fn read_jira(self) -> Jira {
        let persistence = self.file.read_persistence();

        let persistence_state = match persistence {
            Ok(state) => state,
            Err(e) => panic!("Unexpected read error {}", e),
        };

        persistence_state.jira
    }

    pub fn write_jira(self, tickets: &[Ticket]) -> Result<Jira, Box<dyn Error>> {
        let mut persistence: Persistence = self.file.read_persistence()?;
        persistence.jira.tickets = tickets.to_owned();
        Ok(self.file.write_persistence(persistence)?.jira)
    }
}

pub struct PersistenceFile {
    file_path: PathBuf,
}

impl PersistenceFile {
    pub fn new() -> PersistenceFile {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        PersistenceFile {
            file_path: home_dir.join(".devtool").join("persistence.yaml"),
        }
    }

    #[cfg(test)]
    pub fn new_from_path(file_path: PathBuf) -> PersistenceFile {
        PersistenceFile { file_path }
    }
    fn write_persistence(&self, persistence: Persistence) -> Result<Persistence, Box<dyn Error>> {
        let yaml_string = serde_yaml::to_string(&persistence)?;

        fs::write(&self.file_path, yaml_string)?;

        Ok(persistence)
    }

    fn read_persistence(&self) -> Result<Persistence, Box<dyn Error>> {
        let persistence_yaml = match fs::read_to_string(&self.file_path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let default = Persistence { jira: Jira::new() };
                self.write_persistence(default.clone())
                    .expect("Failed to write persistence");

                serde_yaml::to_string(&default)?
            }
            Err(e) => return Err(Box::new(e)),
        };

        // Parse YAML into Persistence struct
        let parsed: Persistence = serde_yaml::from_str(&persistence_yaml)?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_persistence_path(dir: &TempDir) -> PathBuf {
        dir.path().join("persistence.yaml")
    }

    fn sample_ticket() -> Ticket {
        Ticket::new(
            "TEST-1".to_string(),
            "Test ticket".to_string(),
            "In Progress".to_string(),
            "Alice".to_string(),
        )
    }

    #[test]
    fn read_persistence_creates_new_file_when_it_does_not_already_exist() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let file = PersistenceFile::new_from_path(path.clone());
        let persistence = file.read_persistence().unwrap();

        assert!(persistence.jira.tickets.is_empty());
        assert!(path.exists());
    }

    #[test]
    fn write_then_read_persistence_returns_expected() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let file = PersistenceFile::new_from_path(path.clone());
        let persistence = Persistence {
            jira: Jira {
                tickets: vec![sample_ticket()],
            },
        };
        file.write_persistence(persistence).unwrap();

        let file = PersistenceFile::new_from_path(path);
        let actual = file.read_persistence().unwrap();

        assert_eq!(actual.jira.tickets[0].assignee, "Alice");
        assert_eq!(actual.jira.tickets[0].status, "In Progress");
        assert_eq!(actual.jira.tickets[0].id, "TEST-1");
        assert_eq!(actual.jira.tickets[0].title, "Test ticket");
    }

    #[test]
    fn read_jira_returns_empty_with_new_file() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let jira = JiraFile::new_from_path(path.clone()).read_jira();

        assert!(jira.tickets.is_empty());
    }

    #[test]
    fn write_jira_returns_written_ticket_for_new_file() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let jira = JiraFile::new_from_path(path.clone()).write_jira(&vec![sample_ticket()]);

        assert_eq!(jira.unwrap().tickets.is_empty(), false);
    }

    #[test]
    fn write_jira_empties_when_saving_no_tickets() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let jira_file = JiraFile::new_from_path(path.clone());
        let saved_jira = jira_file.write_jira(&vec![sample_ticket()]);

        assert_eq!(saved_jira.unwrap().tickets.is_empty(), false);

        let jira_file = JiraFile::new_from_path(path.clone());
        let empty_jira = jira_file.write_jira(&vec![]);

        assert!(empty_jira.unwrap().tickets.is_empty());
    }

    #[test]
    fn write_jira_overwrites_existing_ticket() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let jira_file = JiraFile::new_from_path(path.clone());
        let saved_jira = jira_file.write_jira(&vec![sample_ticket()]);

        assert_eq!(saved_jira.unwrap().tickets.is_empty(), false);

        let overwritten_ticket = Ticket {
            id: String::from("OVER-1"),
            title: String::from("OVERWRITTEN"),
            assignee: String::from("John"),
            status: String::from("COMPLETED"),
        };

        let jira_file = JiraFile::new_from_path(path.clone());
        let overwritten_jira = jira_file.write_jira(&vec![overwritten_ticket]);

        let ticket = overwritten_jira.unwrap().tickets;
        assert_eq!(ticket.is_empty(), false);
        assert_eq!(ticket[0].id, "OVER-1");
        assert_eq!(ticket[0].title, "OVERWRITTEN");
        assert_eq!(ticket[0].assignee, "John");
        assert_eq!(ticket[0].status, "COMPLETED");
    }

    #[test]
    fn write_jira_saves_multiple_tickets() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let jira_file = JiraFile::new_from_path(path.clone());
        let second_ticket = Ticket {
            id: "TEST-2".to_string(),
            title: "Second Ticket".to_string(),
            status: "Breaking prod".to_string(),
            assignee: "Not me".to_string(),
        };
        let saved_jira = jira_file.write_jira(&vec![sample_ticket(), second_ticket]);
        let saved_tickets = saved_jira.unwrap().tickets;
        assert_eq!(saved_tickets.iter().count(), 2);
        assert_eq!(saved_tickets[0].assignee, "Alice");
        assert_eq!(saved_tickets[0].status, "In Progress");
        assert_eq!(saved_tickets[0].id, "TEST-1");
        assert_eq!(saved_tickets[0].title, "Test ticket");
        assert_eq!(saved_tickets[1].assignee, "Not me");
        assert_eq!(saved_tickets[1].status, "Breaking prod");
        assert_eq!(saved_tickets[1].id, "TEST-2");
        assert_eq!(saved_tickets[1].title, "Second Ticket");
    }
}
