use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct TicketResponse {
    pub key: String,
    pub fields: Fields,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Fields {
    pub assignee: Option<Assignee>,
    pub status: Status,
    pub summary: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Assignee {
    pub display_name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_ticket_response() {
        let json = r#"{
            "key": "PROJ-123",
            "fields": {
                "assignee": { "displayName": "Alice" },
                "status": { "name": "Done" },
                "summary": "A ticket"
            }
        }"#;
        let ticket: TicketResponse = serde_json::from_str(json).unwrap();
        assert_eq!(ticket.key, "PROJ-123");
        assert_eq!(ticket.fields.summary, "A ticket");
        assert_eq!(ticket.fields.status.name, "Done");
        assert_eq!(ticket.fields.assignee.unwrap().display_name, "Alice");
    }

    #[test]
    fn deserialize_ticket_with_null_assignee() {
        let json = r#"{
            "key": "PROJ-456",
            "fields": {
                "assignee": null,
                "status": { "name": "Open" },
                "summary": "Unassigned ticket"
            }
        }"#;
        let ticket: TicketResponse = serde_json::from_str(json).unwrap();
        assert!(ticket.fields.assignee.is_none());
    }

    #[test]
    fn assignee_uses_camel_case_rename() {
        let json = r#"{"displayName": "Bob"}"#;
        let assignee: Assignee = serde_json::from_str(json).unwrap();
        assert_eq!(assignee.display_name, "Bob");
    }
}
