use crate::client::jira_client::TicketResponse;
use crate::events::event::Direction;
use crate::persistence::{read_jira_persistence, write_jira_tickets};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Jira {
    pub tickets: Vec<Ticket>,
    pub list_state: ListState,
    pub new_ticket_popup: bool,
    pub new_ticket_id: Option<String>,
}

impl Jira {
    pub fn new() -> Jira {
        Self {
            tickets: read_jira_persistence().tickets,
            list_state: ListState::default().with_selected(None),
            new_ticket_popup: false,
            new_ticket_id: None,
        }
    }

    pub fn add_char_to_ticket_id(&mut self, c: char) {
        self.new_ticket_id
            .get_or_insert_with(String::new)
            .push(c.to_ascii_uppercase());
    }

    pub fn remove_char_from_ticket_id(&mut self) {
        self.new_ticket_id = match &self.new_ticket_id {
            None => None,
            Some(id) => {
                if id.len() > 1 {
                    let mut chars = id.chars();
                    chars.next_back();
                    Some(chars.as_str().to_string())
                } else {
                    None
                }
            }
        }
    }

    pub fn set_new_ticket_id(&mut self, id: Option<String>) {
        self.new_ticket_id = id;
    }

    pub fn set_new_ticket_popup(&mut self, visible: bool) {
        self.new_ticket_popup = visible;
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
    }

    pub fn remove_ticket(&mut self) {
        if let Some(ticket_index) = self.list_state.selected() {
            self.tickets.remove(ticket_index);
            self.persist_tickets()
        }
    }

    pub fn swap_tickets(&mut self, direction: Direction) {
        let mut selected_ticket: Option<Ticket> = None;
        let mut new_index: Option<usize> = None;

        if let Some(ticket_index) = self.list_state.selected() {
            if Direction::Up == direction && ticket_index > 0 {
                selected_ticket = Some(self.tickets.remove(ticket_index));
                new_index = Some(ticket_index.saturating_sub(1));
            }
            if Direction::Down == direction && ticket_index < self.tickets.len() - 1 {
                selected_ticket = Some(self.tickets.remove(ticket_index));
                new_index = Some(ticket_index.saturating_add(1));
            }
        }

        if let Some(index) = new_index
            && let Some(ticket) = selected_ticket
        {
            self.tickets.insert(index, ticket);
            self.list_state.select(Some(index));
            self.persist_tickets()
        }
    }

    fn persist_tickets(&mut self) {
        write_jira_tickets(&self.tickets).expect("Failed to persist tickets");
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
