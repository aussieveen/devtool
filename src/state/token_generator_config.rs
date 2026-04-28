use crate::config::model::{Auth0Config, Credentials, ServiceConfig};
use crate::environment::Environment;
use ratatui::widgets::TableState;

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
    pub local: String,
    pub staging: String,
    pub preprod: String,
    pub prod: String,
    pub active_field: Auth0Field,
}

impl Auth0Form {
    pub fn from_existing(config: &Auth0Config) -> Self {
        Self {
            local: config.local.clone(),
            staging: config.staging.clone(),
            preprod: config.preproduction.clone(),
            prod: config.production.clone(),
            active_field: Auth0Field::Local,
        }
    }

    pub fn active_field_value_mut(&mut self) -> &mut String {
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
    pub name: String,
    pub audience: String,
    pub local_id: String,
    pub local_secret: String,
    pub staging_id: String,
    pub staging_secret: String,
    pub preprod_id: String,
    pub preprod_secret: String,
    pub prod_id: String,
    pub prod_secret: String,
    pub active_field: ServiceField,
    /// Some(idx) = editing existing; None = adding new.
    pub edit_index: Option<usize>,
}

impl ServiceForm {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            audience: String::new(),
            local_id: String::new(),
            local_secret: String::new(),
            staging_id: String::new(),
            staging_secret: String::new(),
            preprod_id: String::new(),
            preprod_secret: String::new(),
            prod_id: String::new(),
            prod_secret: String::new(),
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
            name: svc.name.clone(),
            audience: svc.audience.clone(),
            local_id,
            local_secret,
            staging_id,
            staging_secret,
            preprod_id,
            preprod_secret,
            prod_id,
            prod_secret,
            active_field: ServiceField::Name,
            edit_index: Some(idx),
        }
    }

    pub fn active_field_value_mut(&mut self) -> &mut String {
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
        !self.name.trim().is_empty()
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
            if !id.trim().is_empty() || !secret.trim().is_empty() {
                creds.push(Credentials {
                    env,
                    client_id: id.trim().to_string(),
                    client_secret: secret.trim().to_string(),
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
    Service(ServiceForm),
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
        self.form = Some(ActiveEdit::Service(ServiceForm::new()));
    }

    pub fn open_edit_service_form(&mut self, idx: usize, svc: &ServiceConfig) {
        self.form = Some(ActiveEdit::Service(ServiceForm::from_existing(idx, svc)));
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
        form.name = "svc".to_string();
        form.staging_id = "id".to_string();
        form.staging_secret = "sec".to_string();

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
        assert_eq!(form.name, "my-svc");
        assert_eq!(form.staging_id, "cid");
        assert_eq!(form.local_id, "");
        assert_eq!(form.edit_index, Some(0));
    }
}
