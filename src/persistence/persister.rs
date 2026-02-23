use crate::error::model::PersistenceError;
use crate::persistence::model::{Jira, Persistence};
use crate::state::jira::Ticket;
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

#[derive(Clone)]
pub struct JiraFile {
    file: PersistenceFile,
}

impl JiraFile {
    pub fn default() -> JiraFile {
        JiraFile {
            file: PersistenceFile::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_from_path(file_path: PathBuf) -> JiraFile {
        JiraFile {
            file: PersistenceFile::new_from_path(file_path),
        }
    }

    pub fn read_jira(self) -> Result<Jira, PersistenceError> {
        self.file.read_persistence().map(|p| p.jira)
    }

    pub fn write_jira(self, tickets: &[Ticket]) -> Result<(), PersistenceError> {
        let mut persistence: Persistence = self.file.read_persistence()?;
        persistence.jira.tickets = tickets.to_owned();
        self.file.write_persistence(persistence)
    }
}

#[derive(Clone)]
pub struct PersistenceFile {
    file_path: PathBuf,
}

impl PersistenceFile {
    pub fn default() -> PersistenceFile {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        PersistenceFile {
            file_path: home_dir.join(".devtool").join("persistence.yaml"),
        }
    }

    #[cfg(test)]
    pub fn new_from_path(file_path: PathBuf) -> PersistenceFile {
        PersistenceFile { file_path }
    }
    fn write_persistence(&self, persistence: Persistence) -> Result<(), PersistenceError> {
        let yaml_string = serde_yaml::to_string(&persistence)?;
        fs::write(&self.file_path, yaml_string)?;
        Ok(())
    }

    fn read_persistence(&self) -> Result<Persistence, PersistenceError> {
        let persistence_yaml = match fs::read_to_string(&self.file_path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let default = Persistence { jira: Jira::new() };
                self.write_persistence(default.clone())?;
                return Ok(default);
            }
            Err(e) => return Err(PersistenceError::Io(e)),
        };

        // Parse YAML into Persistence struct
        Ok(serde_yaml::from_str(&persistence_yaml)?)
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

        let jira = JiraFile::new_from_path(path.clone()).read_jira().unwrap();

        assert!(jira.tickets.is_empty());
    }

    #[test]
    fn write_jira_saves_ticket_for_new_file() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        JiraFile::new_from_path(path.clone())
            .write_jira(&[sample_ticket()])
            .unwrap();

        let saved = JiraFile::new_from_path(path).read_jira().unwrap();
        assert!(!saved.tickets.is_empty());
    }

    #[test]
    fn write_jira_empties_when_saving_no_tickets() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        JiraFile::new_from_path(path.clone())
            .write_jira(&vec![sample_ticket()])
            .unwrap();

        let saved_jira = JiraFile::new_from_path(path.clone()).read_jira().unwrap();
        assert!(!saved_jira.tickets.is_empty());

        JiraFile::new_from_path(path.clone())
            .write_jira(&vec![])
            .unwrap();
        let empty_jira = JiraFile::new_from_path(path.clone()).read_jira().unwrap();
        assert!(empty_jira.tickets.is_empty());
    }

    #[test]
    fn write_jira_overwrites_existing_ticket() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        JiraFile::new_from_path(path.clone())
            .write_jira(&vec![sample_ticket()])
            .unwrap();

        assert!(
            !JiraFile::new_from_path(path.clone())
                .read_jira()
                .unwrap()
                .tickets
                .is_empty()
        );

        let overwritten_ticket = Ticket {
            id: String::from("OVER-1"),
            title: String::from("OVERWRITTEN"),
            assignee: String::from("John"),
            status: String::from("COMPLETED"),
        };

        JiraFile::new_from_path(path.clone())
            .write_jira(&vec![overwritten_ticket])
            .unwrap();

        let overwritten_jira = JiraFile::new_from_path(path.clone()).read_jira().unwrap();
        let ticket = overwritten_jira.tickets;
        assert!(!ticket.is_empty());
        assert_eq!(ticket[0].id, "OVER-1");
        assert_eq!(ticket[0].title, "OVERWRITTEN");
        assert_eq!(ticket[0].assignee, "John");
        assert_eq!(ticket[0].status, "COMPLETED");
    }

    #[test]
    fn write_jira_saves_multiple_tickets() {
        let dir = TempDir::new().unwrap();
        let path = temp_persistence_path(&dir);

        let second_ticket = Ticket {
            id: "TEST-2".to_string(),
            title: "Second Ticket".to_string(),
            status: "Breaking prod".to_string(),
            assignee: "Not me".to_string(),
        };
        JiraFile::new_from_path(path.clone())
            .write_jira(&vec![sample_ticket(), second_ticket])
            .unwrap();

        let saved_tickets = JiraFile::new_from_path(path.clone())
            .read_jira()
            .unwrap()
            .tickets;

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
