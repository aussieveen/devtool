#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub title: String,
    pub originating_event: String,
    pub tool: String,
    pub description: String,
}
