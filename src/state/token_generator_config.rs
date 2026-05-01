use crate::config::model::{Auth0Config, Credentials, ServiceConfig};
use crate::environment::Environment;
use ratatui::widgets::TableState;
use tui_text_field::TextField;

// ── Field enums ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Auth0Field {
    Local,
    Staging,
    Preprod,
    Prod,
}

impl Auth0Field {
    pub fn next(self) -> Self {
        match self {
            Self::Local => Self::Staging,
            Self::Staging => Self::Preprod,
            Self::Preprod => Self::Prod,
            Self::Prod => Self::Local,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Local => Self::Prod,
            Self::Staging => Self::Local,
            Self::Preprod => Self::Staging,
            Self::Prod => Self::Preprod,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ServiceField {
    Name,
    Audience,
    LocalClientId,
    LocalClientSecret,
    StagingClientId,
    StagingClientSecret,
    PreprodClientId,
    PreprodClientSecret,
    ProdClientId,
    ProdClientSecret,
}

impl ServiceField {
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Audience,
            Self::Audience => Self::LocalClientId,
            Self::LocalClientId => Self::LocalClientSecret,
            Self::LocalClientSecret => Self::StagingClientId,
            Self::StagingClientId => Self::StagingClientSecret,
            Self::StagingClientSecret => Self::PreprodClientId,
            Self::PreprodClientId => Self::PreprodClientSecret,
            Self::PreprodClientSecret => Self::ProdClientId,
            Self::ProdClientId => Self::ProdClientSecret,
            Self::ProdClientSecret => Self::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::ProdClientSecret,
            Self::Audience => Self::Name,
            Self::LocalClientId => Self::Audience,
            Self::LocalClientSecret => Self::LocalClientId,
            Self::StagingClientId => Self::LocalClientSecret,
            Self::StagingClientSecret => Self::StagingClientId,
            Self::PreprodClientId => Self::StagingClientSecret,
            Self::PreprodClientSecret => Self::PreprodClientId,
            Self::ProdClientId => Self::PreprodClientSecret,
            Self::ProdClientSecret => Self::ProdClientId,
        }
    }
}

// ── Inline edit form structs ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Auth0Form {
    pub local: TextField,
    pub staging: TextField,
    pub preprod: TextField,
    pub prod: TextField,
    pub active_field: Auth0Field,
}

impl Auth0Form {
    pub fn from_existing(config: &Auth0Config) -> Self {
        Self {
            local: TextField::new(config.local.clone()),
            staging: TextField::new(config.staging.clone()),
            preprod: TextField::new(config.preproduction.clone()),
            prod: TextField::new(config.production.clone()),
            active_field: Auth0Field::Local,
        }
    }

    pub fn active_field(&self) -> &TextField {
        match self.active_field {
            Auth0Field::Local => &self.local,
            Auth0Field::Staging => &self.staging,
            Auth0Field::Preprod => &self.preprod,
            Auth0Field::Prod => &self.prod,
        }
    }

    pub fn active_field_mut(&mut self) -> &mut TextField {
        match self.active_field {
            Auth0Field::Local => &mut self.local,
            Auth0Field::Staging => &mut self.staging,
            Auth0Field::Preprod => &mut self.preprod,
            Auth0Field::Prod => &mut self.prod,
        }
    }
}

#[derive(Clone)]
pub struct ServiceForm {
    pub name: TextField,
    pub audience: TextField,
    pub local_id: TextField,
    pub local_secret: TextField,
    pub staging_id: TextField,
    pub staging_secret: TextField,
    pub preprod_id: TextField,
    pub preprod_secret: TextField,
    pub prod_id: TextField,
    pub prod_secret: TextField,
    pub active_field: ServiceField,
    /// Some(idx) = editing existing; None = adding new.
    pub edit_index: Option<usize>,
}

impl ServiceForm {
    pub fn new() -> Self {
        Self {
            name: TextField::empty(),
            audience: TextField::empty(),
            local_id: TextField::empty(),
            local_secret: TextField::empty(),
            staging_id: TextField::empty(),
            staging_secret: TextField::empty(),
            preprod_id: TextField::empty(),
            preprod_secret: TextField::empty(),
            prod_id: TextField::empty(),
            prod_secret: TextField::empty(),
            active_field: ServiceField::Name,
            edit_index: None,
        }
    }

    pub fn from_existing(idx: usize, svc: &ServiceConfig) -> Self {
        let creds = |env: Environment| -> (String, String) {
            svc.credentials
                .iter()
                .find(|c| c.env == env)
                .map(|c| (c.client_id.clone(), c.client_secret.clone()))
                .unwrap_or_default()
        };
        let (local_id, local_secret) = creds(Environment::Local);
        let (staging_id, staging_secret) = creds(Environment::Staging);
        let (preprod_id, preprod_secret) = creds(Environment::Preproduction);
        let (prod_id, prod_secret) = creds(Environment::Production);
        Self {
            name: TextField::new(svc.name.clone()),
            audience: TextField::new(svc.audience.clone()),
            local_id: TextField::new(local_id),
            local_secret: TextField::new(local_secret),
            staging_id: TextField::new(staging_id),
            staging_secret: TextField::new(staging_secret),
            preprod_id: TextField::new(preprod_id),
            preprod_secret: TextField::new(preprod_secret),
            prod_id: TextField::new(prod_id),
            prod_secret: TextField::new(prod_secret),
            active_field: ServiceField::Name,
            edit_index: Some(idx),
        }
    }

    pub fn active_field(&self) -> &TextField {
        match self.active_field {
            ServiceField::Name => &self.name,
            ServiceField::Audience => &self.audience,
            ServiceField::LocalClientId => &self.local_id,
            ServiceField::LocalClientSecret => &self.local_secret,
            ServiceField::StagingClientId => &self.staging_id,
            ServiceField::StagingClientSecret => &self.staging_secret,
            ServiceField::PreprodClientId => &self.preprod_id,
            ServiceField::PreprodClientSecret => &self.preprod_secret,
            ServiceField::ProdClientId => &self.prod_id,
            ServiceField::ProdClientSecret => &self.prod_secret,
        }
    }

    pub fn active_field_mut(&mut self) -> &mut TextField {
        match self.active_field {
            ServiceField::Name => &mut self.name,
            ServiceField::Audience => &mut self.audience,
            ServiceField::LocalClientId => &mut self.local_id,
            ServiceField::LocalClientSecret => &mut self.local_secret,
            ServiceField::StagingClientId => &mut self.staging_id,
            ServiceField::StagingClientSecret => &mut self.staging_secret,
            ServiceField::PreprodClientId => &mut self.preprod_id,
            ServiceField::PreprodClientSecret => &mut self.preprod_secret,
            ServiceField::ProdClientId => &mut self.prod_id,
            ServiceField::ProdClientSecret => &mut self.prod_secret,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.value().trim().is_empty()
    }

    /// Build the credentials vec, omitting environments where both fields are empty.
    pub fn to_credentials(&self) -> Vec<Credentials> {
        let mut creds = Vec::new();
        let pairs = [
            (Environment::Local, &self.local_id, &self.local_secret),
            (Environment::Staging, &self.staging_id, &self.staging_secret),
            (
                Environment::Preproduction,
                &self.preprod_id,
                &self.preprod_secret,
            ),
            (Environment::Production, &self.prod_id, &self.prod_secret),
        ];
        for (env, id, secret) in pairs {
            if !id.value().trim().is_empty() || !secret.value().trim().is_empty() {
                creds.push(Credentials {
                    env,
                    client_id: id.value().trim().to_string(),
                    client_secret: secret.value().trim().to_string(),
                });
            }
        }
        creds
    }
}

// ── Config focus ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConfigFocus {
    Auth0,
    Services,
}

// ── Editor ────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub enum ActiveEdit {
    Auth0(Auth0Form),
    Service(Box<ServiceForm>),
}

#[derive(Clone)]
pub struct TokenGeneratorConfigEditor {
    pub table_state: TableState,
    pub form: Option<ActiveEdit>,
    pub config_focus: ConfigFocus,
}

impl TokenGeneratorConfigEditor {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            form: None,
            config_focus: ConfigFocus::Auth0,
        }
    }

    pub fn open_auth0_form(&mut self, config: &Auth0Config) {
        self.form = Some(ActiveEdit::Auth0(Auth0Form::from_existing(config)));
    }

    pub fn open_add_service_form(&mut self) {
        self.form = Some(ActiveEdit::Service(Box::new(ServiceForm::new())));
    }

    pub fn open_edit_service_form(&mut self, idx: usize, svc: &ServiceConfig) {
        self.form = Some(ActiveEdit::Service(Box::new(ServiceForm::from_existing(idx, svc))));
    }

    pub fn has_open_form(&self) -> bool {
        self.form.is_some()
    }

    pub fn close_form(&mut self) {
        self.form = None;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth0_field_next_wraps() {
        assert_eq!(Auth0Field::Prod.next(), Auth0Field::Local);
        assert_eq!(Auth0Field::Local.next(), Auth0Field::Staging);
    }

    #[test]
    fn auth0_field_prev_wraps() {
        assert_eq!(Auth0Field::Local.prev(), Auth0Field::Prod);
        assert_eq!(Auth0Field::Staging.prev(), Auth0Field::Local);
    }

    #[test]
    fn service_field_next_wraps() {
        assert_eq!(ServiceField::ProdClientSecret.next(), ServiceField::Name);
        assert_eq!(ServiceField::Name.next(), ServiceField::Audience);
    }

    #[test]
    fn service_field_prev_wraps() {
        assert_eq!(ServiceField::Name.prev(), ServiceField::ProdClientSecret);
        assert_eq!(ServiceField::Audience.prev(), ServiceField::Name);
    }

    #[test]
    fn service_form_to_credentials_omits_empty_envs() {
        let mut form = ServiceForm::new();
        form.name = TextField::new("svc".to_string());
        form.staging_id = TextField::new("id".to_string());
        form.staging_secret = TextField::new("sec".to_string());

        let creds = form.to_credentials();
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0].env, Environment::Staging);
    }

    #[test]
    fn service_form_is_invalid_when_name_empty() {
        let form = ServiceForm::new();
        assert!(!form.is_valid());
    }

    #[test]
    fn service_form_from_existing_populates_fields() {
        let svc = ServiceConfig {
            name: "my-svc".to_string(),
            audience: "https://api".to_string(),
            credentials: vec![Credentials {
                env: Environment::Staging,
                client_id: "cid".to_string(),
                client_secret: "csec".to_string(),
            }],
        };
        let form = ServiceForm::from_existing(0, &svc);
        assert_eq!(form.name.value(), "my-svc");
        assert_eq!(form.staging_id.value(), "cid");
        assert_eq!(form.local_id.value(), "");
        assert_eq!(form.edit_index, Some(0));
    }
}
