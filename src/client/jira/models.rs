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
    pub description: Option<Description>
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

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Description {
    pub content: Vec<Content>
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Content {
    pub r#type: String,
    pub text: Option<String>,
    pub content: Option<Vec<Content>>,
    pub marks: Option<Vec<Content>>,
    pub attrs: Option<Attributes>
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Attributes{
    pub url: Option<String>
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error_messages: Vec<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum JiraResponse {
    TicketResponse(TicketResponse),
    ErrorResponse(ErrorResponse),
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
                "summary": "A ticket",
                "description": {
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [
                                {
                                    "type": "text",
                                    "text": "this is some text",
                                    "attrs": {
                                        "url": "https://someurl.com"
                                    },
                                    "marks": [
                                        {
                                            "type": "strong"
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;
        let ticket: TicketResponse = serde_json::from_str(json).unwrap();
        assert_eq!(ticket, TicketResponse{
            key: "PROJ-123".to_string(),
            fields: Fields {
                assignee: Some(Assignee{ display_name: "Alice".to_string() }),
                status: Status { name: "Done".to_string() },
                summary: "A ticket".to_string(),
                description: Some(Description{ content: vec![
                    Content{
                        r#type: "paragraph".to_string(),
                        text: None,
                        content: Some(vec![Content{
                            r#type: "text".to_string(),
                            text: Some("this is some text".to_string()),
                            content: None,
                            marks: Some(vec![Content{
                                r#type: "strong".to_string(),
                                text: None,
                                content: None,
                                marks: None,
                                attrs: None,
                            }]),
                            attrs: Some(Attributes{ url: Some("https://someurl.com".to_string()) }),
                        }]),
                        marks: None,
                        attrs: None,
                    }
                ] }),
            },
        });
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
