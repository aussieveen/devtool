use crate::client::jira::models::TicketResponse;
use crate::events::event::Direction;
use crate::persistence::persister::JiraFile;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Jira {
    pub tickets: Vec<Ticket>,
    pub list_state: ListState,
    pub new_ticket_popup: bool,
    pub new_ticket_id: Option<String>,
    pub jira_file: JiraFile,
}

impl Jira {
    pub fn new() -> Jira {
        let jira_file = JiraFile::default();
        let tickets = jira_file.clone().read_jira().tickets;
        Self {
            tickets,
            list_state: ListState::default().with_selected(None),
            new_ticket_popup: false,
            new_ticket_id: None,
            jira_file,
        }
    }

    #[cfg(test)]
    pub fn new_empty(jira_file: JiraFile) -> Jira {
        Self {
            tickets: Vec::new(),
            list_state: ListState::default().with_selected(None),
            new_ticket_popup: false,
            new_ticket_id: None,
            jira_file,
        }
    }

    pub fn add_char_to_ticket_id(&mut self, c: char) {
        self.new_ticket_id
            .get_or_insert_with(String::new)
            .push(c.to_ascii_uppercase());
    }

    pub fn remove_char_from_ticket_id(&mut self) {
        if let Some(id) = &mut self.new_ticket_id {
            id.pop();
            if id.is_empty() {
                self.new_ticket_id = None;
            }
        }
    }

    pub fn add_ticket(&mut self, ticket: TicketResponse) {
        self.tickets.push(Ticket::new(
            ticket.key,
            ticket.fields.summary,
            ticket.fields.status.name,
            match ticket.fields.assignee {
                Some(assignee) => assignee.display_name,
                None => "Unassigned".to_string(),
            },
        ));
        self.persist_tickets();
    }

    pub fn remove_ticket(&mut self) {
        if let Some(ticket_index) = self.list_state.selected() {
            self.tickets.remove(ticket_index);
            self.persist_tickets()
        }
    }

    pub fn swap_tickets(&mut self, direction: Direction) {
        if let Some(ticket_index) = self.list_state.selected() {
            let new_index = match direction {
                Direction::Up if ticket_index > 0 => ticket_index - 1,
                Direction::Down if ticket_index < self.tickets.len() - 1 => ticket_index + 1,
                _ => return,
            };
            let ticket = self.tickets.remove(ticket_index);
            self.tickets.insert(new_index, ticket);
            self.list_state.select(Some(new_index));
            self.persist_tickets();
        }
    }

    fn persist_tickets(&mut self) {
        self.jira_file
            .clone()
            .write_jira(&self.tickets)
            .expect("Failed to persist tickets");
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: String,
    pub assignee: String,
}

impl Ticket {
    pub fn new(id: String, title: String, status: String, assignee: String) -> Ticket {
        Self {
            id,
            title,
            status,
            assignee,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::jira::models::{Assignee, Fields, Status, TicketResponse};
    use crate::events::event::Direction;
    use crate::persistence::persister::JiraFile;
    use crate::state::jira::{Jira, Ticket};
    use std::path::PathBuf;
    use tempfile::TempDir;
    use test_case::test_case;

    fn get_jira_with_path(path: PathBuf) -> Jira {
        Jira {
            tickets: vec![
                Ticket {
                    id: "1".to_string(),
                    title: "title 1".to_string(),
                    status: "in progress".to_string(),
                    assignee: "john".to_string(),
                },
                Ticket {
                    id: "2".to_string(),
                    title: "title 2".to_string(),
                    status: "complete".to_string(),
                    assignee: "jane".to_string(),
                },
            ],
            list_state: Default::default(),
            new_ticket_popup: false,
            new_ticket_id: None,
            jira_file: JiraFile::new_from_path(path),
        }
    }

    fn temp_file_path(dir: &TempDir) -> PathBuf {
        dir.path().join("jirafile.yaml")
    }

    #[test_case(None, "Unassigned"; "Unassigned set as default")]
    #[test_case(Some(Assignee{display_name:"John Smith".to_string()}), "John Smith"; "Assignee set from response")]
    fn jira_add_ticket(assignee: Option<Assignee>, assignee_value: &str) {
        let dir = TempDir::new().unwrap();
        let file_path = temp_file_path(&dir);

        let mut jira = get_jira_with_path(file_path);
        jira.add_ticket(TicketResponse {
            key: "TEST-1".to_string(),
            fields: Fields {
                assignee,
                status: Status {
                    name: "In Progress".to_string(),
                },
                summary: "Testing".to_string(),
            },
        });

        assert_eq!(jira.tickets.len(), 3);
        assert_eq!(jira.tickets[2].id, "TEST-1");
        assert_eq!(jira.tickets[2].title, "Testing");
        assert_eq!(jira.tickets[2].status, "In Progress");
        assert_eq!(jira.tickets[2].assignee, assignee_value);
    }

    #[test_case(Some(0), 1; "Remove ticket leaving one")]
    #[test_case(None, 2; "Nothing removed when nothing selected")]
    fn jira_remove_ticket(selection: Option<usize>, length: usize) {
        let dir = TempDir::new().unwrap();
        let file_path = temp_file_path(&dir);

        let mut jira = get_jira_with_path(file_path);
        jira.list_state.select(selection);
        jira.remove_ticket();

        assert_eq!(jira.tickets.len(), length);
    }

    #[test]
    fn jira_add_char_to_ticket_id_adds_char() {
        let dir = TempDir::new().unwrap();
        let file_path = temp_file_path(&dir);

        let mut jira = get_jira_with_path(file_path);
        jira.add_char_to_ticket_id('s');
        assert_eq!(jira.new_ticket_id.clone().unwrap(), "S");

        jira.add_char_to_ticket_id('-');
        assert_eq!(jira.new_ticket_id.clone().unwrap(), "S-");
    }

    #[test_case(None, None; "String not changed when NONE")]
    #[test_case(Some(String::from("S")), None; "Removing last character set ticket_id to NONE")]
    #[test_case(Some(String::from("SE")), Some(String::from("S")); "Removes a single character")]
    fn jira_remove_char_from_ticket_id(current: Option<String>, expected: Option<String>) {
        let dir = TempDir::new().unwrap();
        let file_path = temp_file_path(&dir);

        let mut jira = get_jira_with_path(file_path);
        jira.new_ticket_id = current;
        jira.remove_char_from_ticket_id();
        assert_eq!(jira.new_ticket_id, expected);
    }

    #[test]
    fn jira_swap_tickets_does_nothing_when_moving_top_ticket_up() {
        let dir = TempDir::new().unwrap();
        let file_path = temp_file_path(&dir);

        let mut jira = get_jira_with_path(file_path);
        jira.list_state.select(Some(0));

        jira.swap_tickets(Direction::Up);
        assert_eq!(
            jira.tickets,
            vec![
                Ticket {
                    id: "1".to_string(),
                    title: "title 1".to_string(),
                    status: "in progress".to_string(),
                    assignee: "john".to_string(),
                },
                Ticket {
                    id: "2".to_string(),
                    title: "title 2".to_string(),
                    status: "complete".to_string(),
                    assignee: "jane".to_string(),
                }
            ]
        )
    }

    #[test]
    fn jira_swap_tickets_does_nothing_when_moving_bottom_ticket_down() {
        let dir = TempDir::new().unwrap();
        let file_path = temp_file_path(&dir);

        let mut jira = get_jira_with_path(file_path);
        jira.list_state.select(Some(1));

        jira.swap_tickets(Direction::Down);
        assert_eq!(
            jira.tickets,
            vec![
                Ticket {
                    id: "1".to_string(),
                    title: "title 1".to_string(),
                    status: "in progress".to_string(),
                    assignee: "john".to_string(),
                },
                Ticket {
                    id: "2".to_string(),
                    title: "title 2".to_string(),
                    status: "complete".to_string(),
                    assignee: "jane".to_string(),
                }
            ]
        )
    }

    #[test]
    fn jira_swap_tickets_move_bottom_ticket_up() {
        let dir = TempDir::new().unwrap();
        let mut jira = get_jira_with_path(temp_file_path(&dir));
        jira.list_state.select(Some(1));

        jira.swap_tickets(Direction::Up);
        assert_eq!(
            jira.tickets,
            vec![
                Ticket {
                    id: "2".to_string(),
                    title: "title 2".to_string(),
                    status: "complete".to_string(),
                    assignee: "jane".to_string(),
                },
                Ticket {
                    id: "1".to_string(),
                    title: "title 1".to_string(),
                    status: "in progress".to_string(),
                    assignee: "john".to_string(),
                }
            ]
        )
    }

    #[test]
    fn jira_swap_tickets_move_top_ticket_down() {
        let dir = TempDir::new().unwrap();
        let mut jira = get_jira_with_path(temp_file_path(&dir));
        jira.list_state.select(Some(0));

        jira.swap_tickets(Direction::Down);
        assert_eq!(
            jira.tickets,
            vec![
                Ticket {
                    id: "2".to_string(),
                    title: "title 2".to_string(),
                    status: "complete".to_string(),
                    assignee: "jane".to_string(),
                },
                Ticket {
                    id: "1".to_string(),
                    title: "title 1".to_string(),
                    status: "in progress".to_string(),
                    assignee: "john".to_string(),
                }
            ]
        )
    }
}
