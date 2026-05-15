use crate::client::auth_zero::api::ImmediateAuthZeroApi;
use crate::client::healthcheck::api::ImmediateHealthcheckApi;
use crate::client::jira::api::ImmediateJiraApi;
use crate::config::loader::ConfigFile;
use crate::config::model::Config;
use crate::event::events::AppEvent::*;
use crate::event::events::GenericEvent::{
    CopyToClipboard, OpenInBrowser, Quit, QuitConfirm, SetFocus,
};
use crate::event::events::JiraEvent::ScanTickets;
use crate::event::events::ServiceStatusEvent::Scan;
use crate::event::events::{AppEvent, Event, GenericEvent};
use crate::event::handler::EventHandler;
use crate::event::sender::EventSender;
use crate::input::key_bindings::register_bindings;
use crate::input::key_context::KeyContext;
use crate::input::key_context::KeyContext::{Global, List, Logs, Popup as PopupCtx};
use crate::input::key_event_map::KeyEventMap;
use crate::popup::model::Popup;
pub(crate) use crate::state::app::{AppFocus, Tool};
use crate::state::log::{LogEntry, LogLevel, LogSource};
use crate::tools::context::PluginContext;
use crate::tools::plugin::{create_plugins, Plugin};
use crate::ui::widgets::popup::{Part, Type};
use crate::utils::update_list_state;
use crate::{state::app::AppState, ui::layout, ui::widgets::*};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};
use ratatui::{DefaultTerminal, Frame};
use std::sync::Arc;
use std::time::Duration;

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    pub(crate) state: AppState,
    event_handler: EventHandler,
    pub(crate) event_sender: EventSender,
    pub(crate) config: Config,
    pub(crate) config_loader: ConfigFile,
    key_event_map: KeyEventMap,
    plugins: Vec<Box<dyn Plugin>>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config, config_loader: ConfigFile) -> Self {
        let event_handler = EventHandler::new();
        let event_sender = event_handler.sender();
        let plugins = create_plugins(
            &config,
            Arc::new(ImmediateAuthZeroApi::new()),
            Arc::new(ImmediateJiraApi::new()),
            Arc::new(ImmediateHealthcheckApi::new()),
        );
        Self {
            running: true,
            state: AppState::new(&config),
            event_handler,
            event_sender,
            config,
            config_loader,
            key_event_map: KeyEventMap::default(),
            plugins,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        // Log app startup
        self.state.log.push_log(LogEntry::new(
            LogLevel::Info,
            LogSource::App,
            "App started — config loaded",
        ));

        let async_sender = self.event_sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_mins(15));
            loop {
                interval.tick().await; // This should go first.
                async_sender.send_service_status_event(Scan);
                async_sender.send_jira_event(ScanTickets);
            }
        });

        // Register global/structural bindings, then each plugin's own bindings
        register_bindings(&mut self.key_event_map);
        // Safety: register_bindings borrows key_event_map; plugins is not used there.
        // We need a raw ptr dance here to avoid borrow-checker issues with self.
        // Instead, collect and register in a separate step.
        for plugin in &self.plugins {
            plugin.register_bindings(&mut self.key_event_map);
        }

        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            match self.event_handler.next().await? {
                Event::Tick => {}
                Event::Crossterm(event) => match event {
                    event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(event) => self.handle_app_event(event),
                Event::Generic(event) => self.handle_generic_event(event),
                event => self.dispatch_to_plugins(event),
            }
        }
        Ok(())
    }

    fn dispatch_to_plugins(&mut self, event: Event) {
        let mut ctx = PluginContext {
            config: &mut self.config,
            config_loader: &self.config_loader,
            sender: &self.event_sender,
            popup: &mut self.state.popup,
            focus: &mut self.state.focus,
        };
        for plugin in &mut self.plugins {
            if plugin.handle_event(&event, &mut ctx) {
                break;
            }
        }
    }

    fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            OpenLogs => {
                self.state.focus = AppFocus::Logs;
                if self.state.has_popup() {
                    self.event_sender.send_app_event(DismissPopup);
                    self.state.log.select_logs()
                } else {
                    if self.state.log.selected_item == crate::state::log::LogsItem::Activity {
                        self.state.log.mark_activity_seen();
                    }
                }
            }
            LogsListMove(direction) => {
                use crate::event::events::Direction;
                match direction {
                    Direction::Down => {
                        self.state.log.select_logs();
                    }
                    Direction::Up => {
                        self.state.log.select_activity();
                        if self.state.log.selected_item == crate::state::log::LogsItem::Activity {
                            self.state.log.mark_activity_seen();
                        }
                    }
                }
            }
            ActivityEvent(source, message) => {
                self.state.log.push_activity(source, message);
            }
            AppLog(entry) => {
                let title = entry.title.clone();
                let level = entry.level;
                self.state.log.push_log(entry);
                if level <= LogLevel::Error {
                    self.state.popup = Some(Popup::new(
                        Type::Error,
                        title,
                        vec![
                            Part::Text("See "),
                            Part::Key("3"),
                            Part::Text(" Logs for details  "),
                        ],
                    ));
                }
            }
            ListSelect(tool_state) => self.state.current_tool = tool_state,
            ListMove(direction) => {
                let tool_list = &mut self.state.tool_list;
                update_list_state::update_list(
                    &mut tool_list.list_state,
                    direction,
                    tool_list.items.len(),
                );
                if let Some(index) = tool_list.list_state.selected()
                    && let Some(tool) = tool_list.items.get(index).cloned()
                {
                    self.event_sender.send_app_event(ListSelect(tool))
                }
            }
            DismissPopup => self.state.popup = None,

            // config panel event
            ConfigListMove(direction) => {
                let editor = &mut self.state.config_editor;
                update_list_state::update_list(
                    &mut editor.list_state,
                    direction,
                    editor.items.len(),
                );
            }
            ToggleFeature => {
                let has_jira_config = self.config.jira.is_some();
                if let Some((tool, now_enabled)) = self.state.config_editor.toggle_selected() {
                    let has_min_config = self.plugins.iter()
                        .find(|p| p.id() == tool)
                        .map(|p| p.has_min_config(&self.config))
                        .unwrap_or(false);
                    if now_enabled && !has_min_config {
                        self.state.config_editor.toggle_selected();
                    } else {
                        self.config.features = self.state.config_editor.to_features();
                        let _ = self.config_loader.write_config(&self.config);
                        self.state.rebuild_tool_list(has_jira_config);
                    }
                }
            }
            OpenToolConfig => {
                if let Some(idx) = self.state.config_editor.list_state.selected()
                    && let Some(item) = self.state.config_editor.items.get(idx)
                {
                    self.state.focus = AppFocus::ToolConfig(item.tool);
                }
            }
            RebuildToolList => {
                let has_jira_config = self.config.jira.is_some();
                self.state.config_editor.sync_from_features(&self.config.features);
                self.state.rebuild_tool_list(has_jira_config);
            }
            CloseToolConfig => {
                if let AppFocus::ToolConfig(tool) = self.state.focus {
                    if let Some(plugin) = self.plugins.iter_mut().find(|p| p.id() == tool) {
                        if plugin.has_open_form() {
                            plugin.close_form();
                        } else {
                            self.state.focus = AppFocus::Config;
                        }
                    } else {
                        self.state.focus = AppFocus::Config;
                    }
                } else {
                    self.state.focus = AppFocus::Config;
                }
            }
        }
    }

    fn handle_generic_event(&mut self, event: GenericEvent) {
        match event {
            Quit => {
                self.state.popup = Some(
                    Popup::new(
                        Type::Confirm,
                        "Confirm Quit".to_string(),
                        vec![Part::Key("q"), Part::Text(" again to quit  ")],
                    )
                    .with_action('q', "quit", Event::Generic(QuitConfirm)),
                )
            }
            QuitConfirm => self.running = false,
            SetFocus(focus) => self.state.focus = focus,
            CopyToClipboard | OpenInBrowser => {
                let current = self.state.current_tool;
                let mut ctx = PluginContext {
                    config: &mut self.config,
                    config_loader: &self.config_loader,
                    sender: &self.event_sender,
                    popup: &mut self.state.popup,
                    focus: &mut self.state.focus,
                };
                if let Some(plugin) = self.plugins.iter_mut().find(|p| p.id() == current) {
                    plugin.handle_generic_event(&event, &mut ctx);
                }
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let areas = layout::main(frame.area(), self.state.effective_focus());

        list::render(frame, areas.tools_list, &mut self.state);
        config_list::render(frame, areas.config_list, &mut self.state);
        logs_list::render(frame, areas.logs_list, &mut self.state);

        if matches!(self.state.focus, AppFocus::Logs) {
            tools::logs::render(frame, areas.content, &self.state.log, true);
        } else {
            self.render_content(frame, areas.content);
        }

        footer::render(frame, areas.footer, &self.state, &self.plugins);

        if let Some(popup) = &self.state.popup {
            popup::render(frame, popup)
        }
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::styles;
        let border_style = styles::block_style(styles::tool_has_focus(self.state.effective_focus()));

        // Config preview mode (AppFocus::Config)
        if self.state.effective_focus() == AppFocus::Config {
            if let Some(idx) = self.state.config_editor.list_state.selected()
                && let Some(item) = self.state.config_editor.items.get(idx)
            {
                let tool = item.tool;
                if let Some(plugin) = self.plugins.iter_mut().find(|p| p.id() == tool) {
                    let pane = Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title(plugin.config_title());
                    let inner = pane.inner(area);
                    frame.render_widget(pane, area);
                    plugin.render_config(frame, inner, &self.config);
                    return;
                }
            }
        }

        // ToolConfig mode
        if let AppFocus::ToolConfig(tool) = self.state.effective_focus() {
            if let Some(plugin) = self.plugins.iter_mut().find(|p| p.id() == tool) {
                let pane = Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(plugin.config_title());
                let inner = pane.inner(area);
                frame.render_widget(pane, area);
                plugin.render_config(frame, inner, &self.config);
                return;
            }
        }

        // Normal tool view
        if self.state.tool_list.items.is_empty() {
            use ratatui::prelude::Alignment;
            use ratatui::style::{Color, Style};
            use ratatui::text::{Line, Span};
            use ratatui::widgets::Paragraph;
            let pane = Block::default().borders(Borders::ALL).border_style(border_style);
            let inner = pane.inner(area);
            frame.render_widget(pane, area);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("No tools enabled — press ", Style::default().fg(Color::DarkGray)),
                    Span::styled("[2]", crate::ui::styles::key_style()),
                    Span::styled(" to configure.", Style::default().fg(Color::DarkGray)),
                ]))
                .alignment(Alignment::Center),
                inner,
            );
            return;
        }

        let current = self.state.current_tool;
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.id() == current) {
            let pane = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" {} ", plugin.title()));
            let inner = pane.inner(area);
            frame.render_widget(pane, area);
            plugin.render(frame, inner, &self.config);
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        if self.state.has_popup() {
            if let KeyCode::Char(c) = key.code
                && let Some(popup) = &self.state.popup
                && let Some(action) = popup.actions.iter().find(|a| a.key == c)
            {
                let event = action.event.clone();
                self.event_sender.send_event(event);
            }
            self.state.popup = None;
            return Ok(());
        }
        // First-match-wins: the most specific context in the stack takes priority.
        // This prevents lower-priority contexts (e.g. Global Quit on Esc) from
        // also firing when a higher-priority context already handled the key.
        for context in self.context_stack() {
            if let Some(event) = self.key_event_map.resolve(context, key) {
                self.event_sender.send_event(event.clone());
                break;
            }
        }

        Ok(())
    }

    fn context_stack(&self) -> Vec<KeyContext> {
        let mut stack = Vec::new();

        if self.state.has_popup() {
            stack.push(PopupCtx);
        } else {
            match self.state.focus {
                AppFocus::List => {
                    stack.push(List);
                }
                AppFocus::Tool => {
                    if let Some(plugin) = self.plugins.iter().find(|p| p.id() == self.state.current_tool) {
                        for ctx in plugin.key_contexts() {
                            stack.push(ctx);
                        }
                    }
                }
                AppFocus::ToolConfig(tool) => {
                    if let Some(plugin) = self.plugins.iter().find(|p| p.id() == tool) {
                        for ctx in plugin.config_key_contexts() {
                            stack.push(ctx);
                        }
                    } else {
                        stack.push(KeyContext::ToolConfig(tool));
                    }
                }
                AppFocus::Config => {
                    stack.push(KeyContext::Config);
                }
                AppFocus::Logs => {
                    stack.push(Logs);
                }
                AppFocus::JiraInput => {
                    use crate::input::key_context::KeyContext::Editing;
                    stack.push(Editing(Tool::Jira));
                }
            }
        }

        stack.push(Global);
        stack
    }
}
