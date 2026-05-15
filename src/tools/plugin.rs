use std::sync::Arc;
use ratatui::Frame;
use ratatui::prelude::Rect;
use ratatui::text::Line;
use crate::client::auth_zero::api::AuthZeroApi;
use crate::client::healthcheck::api::HealthcheckApi;
use crate::client::jira::api::JiraApi;
use crate::config::model::{Config, Features};
use crate::event::events::{Event, GenericEvent};
use crate::input::key_context::KeyContext;
use crate::input::key_event_map::KeyEventMap;
use crate::state::tools::Tool;
use crate::tools::context::PluginContext;

pub trait Plugin: Send {
    // ── Identity ────────────────────────────────────────────────────────────
    fn id(&self)           -> Tool;
    fn title(&self)        -> &'static str;
    fn menu_entry(&self)   -> &'static str;
    fn config_title(&self) -> &'static str;

    // ── Feature-flag integration ─────────────────────────────────────────────
    fn has_min_config(&self, config: &Config) -> bool;
    fn is_enabled(&self, features: &Features) -> bool;
    fn apply_feature_flag(&self, features: &mut Features, enabled: bool);

    // ── Key bindings (each plugin registers only its own) ────────────────────
    fn register_bindings(&self, map: &mut KeyEventMap);

    // ── Key contexts for context_stack() ────────────────────────────────────
    fn key_contexts(&self) -> Vec<KeyContext>;
    fn config_key_contexts(&self) -> Vec<KeyContext>;

    // ── Rendering ───────────────────────────────────────────────────────────
    fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config);
    fn render_config(&mut self, frame: &mut Frame, area: Rect, config: &Config);

    // ── Footer hints — each plugin knows its own contextual hint lines ───────
    fn tool_hints(&self) -> (Line<'static>, Line<'static>);
    fn config_hints(&self) -> (Line<'static>, Line<'static>);

    // ── Event handling (return true = consumed, stop iterating) ──────────────
    fn handle_event(&mut self, event: &Event, ctx: &mut PluginContext) -> bool;
    fn handle_generic_event(&mut self, event: &GenericEvent, ctx: &mut PluginContext) -> bool;

    // ── Config editor form state (used by CloseToolConfig) ───────────────────
    fn has_open_form(&self) -> bool;
    fn close_form(&mut self);
}

use crate::tools::{jira, service_status, token_generator};

pub fn create_plugins(
    config: &Config,
    auth_zero_api: Arc<dyn AuthZeroApi>,
    jira_api: Arc<dyn JiraApi>,
    healthcheck_api: Arc<dyn HealthcheckApi>,
) -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(service_status::ServiceStatusPlugin::new(config, healthcheck_api)),
        Box::new(token_generator::TokenGeneratorPlugin::new(config, auth_zero_api)),
        Box::new(jira::JiraPlugin::new(config, jira_api)),
        // ← new tool: one line here
    ]
}