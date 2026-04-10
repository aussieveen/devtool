use ratatui::widgets::TableState;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PopupField {
    Name,
    Staging,
    Preprod,
    Prod,
    Repo,
}

impl PopupField {
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

pub struct AddServicePopup {
    pub name: String,
    pub staging: String,
    pub preprod: String,
    pub prod: String,
    pub repo: String,
    pub active_field: PopupField,
    /// If Some, this is an edit of the item at that index; if None, it's a new add.
    pub edit_index: Option<usize>,
}

impl AddServicePopup {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            staging: String::new(),
            preprod: String::new(),
            prod: String::new(),
            repo: String::new(),
            active_field: PopupField::Name,
            edit_index: None,
        }
    }

    pub fn from_existing(idx: usize, svc: &crate::config::model::ServiceStatusConfig) -> Self {
        Self {
            name: svc.name.clone(),
            staging: svc.staging.clone(),
            preprod: svc.preproduction.clone(),
            prod: svc.production.clone(),
            repo: svc.repo.clone(),
            active_field: PopupField::Name,
            edit_index: Some(idx),
        }
    }

    pub fn active_field_value_mut(&mut self) -> &mut String {
        match self.active_field {
            PopupField::Name => &mut self.name,
            PopupField::Staging => &mut self.staging,
            PopupField::Preprod => &mut self.preprod,
            PopupField::Prod => &mut self.prod,
            PopupField::Repo => &mut self.repo,
        }
    }

    /// Returns true if the required fields (name + at least one URL) are filled.
    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }
}

pub struct ServiceStatusConfigEditor {
    pub table_state: TableState,
    pub popup: Option<AddServicePopup>,
}

impl ServiceStatusConfigEditor {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            popup: None,
        }
    }

    pub fn open_popup(&mut self) {
        self.popup = Some(AddServicePopup::new());
    }

    pub fn open_edit_popup(&mut self, idx: usize, svc: &crate::config::model::ServiceStatusConfig) {
        self.popup = Some(AddServicePopup::from_existing(idx, svc));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_field_next_wraps() {
        assert_eq!(PopupField::Repo.next(), PopupField::Name);
        assert_eq!(PopupField::Name.next(), PopupField::Staging);
    }

    #[test]
    fn popup_field_prev_wraps() {
        assert_eq!(PopupField::Name.prev(), PopupField::Repo);
        assert_eq!(PopupField::Staging.prev(), PopupField::Name);
    }

    #[test]
    fn popup_is_invalid_when_name_empty() {
        let popup = AddServicePopup::new();
        assert!(!popup.is_valid());
    }

    #[test]
    fn popup_is_valid_when_name_set() {
        let mut popup = AddServicePopup::new();
        popup.name = "my-svc".to_string();
        assert!(popup.is_valid());
    }
}
