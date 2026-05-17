use ratatui::widgets::TableState;
use tui_text_field::TextField;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FormField {
    Name,
    Staging,
    Preprod,
    Prod,
    Repo,
}

impl FormField {
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Staging,
            Self::Staging => Self::Preprod,
            Self::Preprod => Self::Prod,
            Self::Prod => Self::Repo,
            Self::Repo => Self::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::Repo,
            Self::Staging => Self::Name,
            Self::Preprod => Self::Staging,
            Self::Prod => Self::Preprod,
            Self::Repo => Self::Prod,
        }
    }
}

#[derive(Clone)]
pub struct AddServiceForm {
    pub name: TextField,
    pub staging: TextField,
    pub preprod: TextField,
    pub prod: TextField,
    pub repo: TextField,
    pub active_field: FormField,
    /// If Some, this is an edit of the item at that index; if None, it's a new add.
    pub edit_index: Option<usize>,
}

impl AddServiceForm {
    pub fn new() -> Self {
        Self {
            name: TextField::empty(),
            staging: TextField::empty(),
            preprod: TextField::empty(),
            prod: TextField::empty(),
            repo: TextField::empty(),
            active_field: FormField::Name,
            edit_index: None,
        }
    }

    pub fn from_existing(idx: usize, svc: &crate::config::model::ServiceStatusConfig) -> Self {
        Self {
            name: TextField::new(svc.name.clone()),
            staging: TextField::new(svc.staging.clone()),
            preprod: TextField::new(svc.preproduction.clone()),
            prod: TextField::new(svc.production.clone()),
            repo: TextField::new(svc.repo.clone()),
            active_field: FormField::Name,
            edit_index: Some(idx),
        }
    }

    pub fn active_field(&self) -> &TextField {
        match self.active_field {
            FormField::Name => &self.name,
            FormField::Staging => &self.staging,
            FormField::Preprod => &self.preprod,
            FormField::Prod => &self.prod,
            FormField::Repo => &self.repo,
        }
    }

    pub fn active_field_mut(&mut self) -> &mut TextField {
        match self.active_field {
            FormField::Name => &mut self.name,
            FormField::Staging => &mut self.staging,
            FormField::Preprod => &mut self.preprod,
            FormField::Prod => &mut self.prod,
            FormField::Repo => &mut self.repo,
        }
    }

    /// Returns true if the required fields (name + at least one URL) are filled.
    pub fn is_valid(&self) -> bool {
        !self.name.value().trim().is_empty()
    }
}

pub struct ServiceStatusConfigEditor {
    pub table_state: TableState,
    pub form: Option<AddServiceForm>,
}

impl ServiceStatusConfigEditor {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            form: None,
        }
    }

    pub fn open_form(&mut self) {
        self.form = Some(AddServiceForm::new());
    }

    pub fn open_edit_form(&mut self, idx: usize, svc: &crate::config::model::ServiceStatusConfig) {
        self.form = Some(AddServiceForm::from_existing(idx, svc));
    }

    pub fn has_open_form(&self) -> bool {
        self.form.is_some()
    }

    pub fn close_form(&mut self) {
        self.form = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_field_next_wraps() {
        assert_eq!(FormField::Repo.next(), FormField::Name);
        assert_eq!(FormField::Name.next(), FormField::Staging);
    }

    #[test]
    fn form_field_prev_wraps() {
        assert_eq!(FormField::Name.prev(), FormField::Repo);
        assert_eq!(FormField::Staging.prev(), FormField::Name);
    }

    #[test]
    fn form_is_invalid_when_name_empty() {
        let form = AddServiceForm::new();
        assert!(!form.is_valid());
    }

    #[test]
    fn form_is_valid_when_name_set() {
        let mut form = AddServiceForm::new();
        form.name = TextField::new("my-svc".to_string());
        assert!(form.is_valid());
    }
}
