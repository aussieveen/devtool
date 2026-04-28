use crate::config::model::JiraConfig;

// ── Field enum ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum JiraField {
    Url,
    Email,
    Token,
}

impl JiraField {
    pub fn next(self) -> Self {
        match self {
            Self::Url => Self::Email,
            Self::Email => Self::Token,
            Self::Token => Self::Url,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Url => Self::Token,
            Self::Email => Self::Url,
            Self::Token => Self::Email,
        }
    }
}

// ── Inline edit form ─────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct JiraConfigForm {
    pub url: String,
    pub email: String,
    pub token: String,
    pub active_field: JiraField,
}

impl JiraConfigForm {
    pub fn from_existing(config: &JiraConfig) -> Self {
        Self {
            url: config.url.clone(),
            email: config.email.clone(),
            token: config.token.clone(),
            active_field: JiraField::Url,
        }
    }

    pub fn empty() -> Self {
        Self {
            url: String::new(),
            email: String::new(),
            token: String::new(),
            active_field: JiraField::Url,
        }
    }

    pub fn active_field_value_mut(&mut self) -> &mut String {
        match self.active_field {
            JiraField::Url => &mut self.url,
            JiraField::Email => &mut self.email,
            JiraField::Token => &mut self.token,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.url.trim().is_empty() && self.email.trim().is_empty() && self.token.trim().is_empty()
    }
}

// ── Editor ────────────────────────────────────────────────────────────────────
pub struct JiraConfigEditor {
    pub form: Option<JiraConfigForm>,
}

impl JiraConfigEditor {
    pub fn new() -> Self {
        Self { form: None }
    }

    pub fn open_form(&mut self, config: Option<&JiraConfig>) {
        self.form = Some(match config {
            Some(c) => JiraConfigForm::from_existing(c),
            None => JiraConfigForm::empty(),
        });
    }

    pub fn has_open_form(&self) -> bool
    {
        self.form.is_some()
    }

    pub fn close_form(&mut self)
    {
        self.form = None;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jira_field_next_wraps() {
        assert_eq!(JiraField::Token.next(), JiraField::Url);
        assert_eq!(JiraField::Url.next(), JiraField::Email);
    }

    #[test]
    fn jira_field_prev_wraps() {
        assert_eq!(JiraField::Url.prev(), JiraField::Token);
        assert_eq!(JiraField::Email.prev(), JiraField::Url);
    }

    #[test]
    fn form_from_existing_populates_fields() {
        let cfg = JiraConfig {
            url: "https://jira.example.com".to_string(),
            email: "user@example.com".to_string(),
            token: "secret".to_string(),
        };
        let form = JiraConfigForm::from_existing(&cfg);
        assert_eq!(form.url, "https://jira.example.com");
        assert_eq!(form.email, "user@example.com");
        assert_eq!(form.token, "secret");
        assert_eq!(form.active_field, JiraField::Url);
    }

    #[test]
    fn form_is_empty_when_all_blank() {
        let form = JiraConfigForm::empty();
        assert!(form.is_empty());
    }

    #[test]
    fn form_is_not_empty_when_url_set() {
        let mut form = JiraConfigForm::empty();
        form.url = "https://jira.example.com".to_string();
        assert!(!form.is_empty());
    }
}
