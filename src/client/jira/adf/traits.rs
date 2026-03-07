pub trait ToMarkdown {
    fn to_markdown(&self) -> String;
}

pub trait Apply {
    fn apply(&self, string: &str) -> String;
}
