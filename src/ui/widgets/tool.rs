use crate::config::model::Config;
use crate::state::app::{AppFocus, AppState, Tool};
use crate::ui::styles;
use crate::ui::widgets::config;
use ratatui::prelude::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &mut AppState,
    config_data: &Config,
) {
    let content_block_border_style =
        styles::block_style(styles::tool_has_focus(state.effective_focus()));

    // ── Config preview mode (AppFocus::Config) ───────────────────────────────
    if state.effective_focus() == AppFocus::Config {
        if let Some(idx) = state.config_editor.list_state.selected() {
            if let Some(item) = state.config_editor.items.get(idx) {
                let tool = item.tool;
                let title = match tool {
                    Tool::ServiceStatus => " Service Status — Config ",
                    Tool::TokenGenerator => " Token Generator — Config ",
                    Tool::Jira => " Jira — Config ",
                };
                let pane = Block::default()
                    .borders(Borders::ALL)
                    .border_style(content_block_border_style)
                    .title(title)
                    .title_alignment(Alignment::Center);
                let inner = pane.inner(area);
                frame.render_widget(pane, area);
                match tool {
                    Tool::ServiceStatus => {
                        config::service_status::render(
                            frame,
                            inner,
                            &mut state.service_status_config_editor,
                            &config_data.servicestatus,
                        );
                    }
                    Tool::TokenGenerator => {
                        config::token_generator::render(
                            frame,
                            inner,
                            &mut state.token_generator_config_editor,
                            &config_data.tokengenerator.auth0,
                            &config_data.tokengenerator.services,
                        );
                    }
                    Tool::Jira => {
                        config::jira::render(
                            frame,
                            inner,
                            &mut state.jira_config_editor,
                            config_data.jira.as_ref(),
                        );
                    }
                }
                return;
            }
        }
    }

    // ── Tool Config mode (AppFocus::ToolConfig) ──────────────────────────────
    if let AppFocus::ToolConfig(tool) = state.effective_focus() {
        let title = match tool {
            Tool::ServiceStatus => " Service Status — Config ",
            Tool::TokenGenerator => " Token Generator — Config ",
            Tool::Jira => " Jira — Config ",
        };
        let pane = Block::default()
            .borders(Borders::ALL)
            .border_style(content_block_border_style)
            .title(title)
            .title_alignment(Alignment::Center);
        let inner = pane.inner(area);
        frame.render_widget(pane, area);

        match tool {
            Tool::ServiceStatus => {
                config::service_status::render(
                    frame,
                    inner,
                    &mut state.service_status_config_editor,
                    &config_data.servicestatus,
                );
            }
            Tool::TokenGenerator => {
                config::token_generator::render(
                    frame,
                    inner,
                    &mut state.token_generator_config_editor,
                    &config_data.tokengenerator.auth0,
                    &config_data.tokengenerator.services,
                );
            }
            Tool::Jira => {
                config::jira::render(
                    frame,
                    inner,
                    &mut state.jira_config_editor,
                    config_data.jira.as_ref(),
                );
            }
        }
        return;
    }

    // ── Normal tool view ─────────────────────────────────────────────────────
    if state.tool_list.items.is_empty() {
        let pane = Block::default()
            .borders(Borders::ALL)
            .border_style(content_block_border_style);
        let inner = pane.inner(area);
        frame.render_widget(pane, area);
        frame.render_widget(
            Paragraph::new(Line::from("No tools enabled — press [2] to configure."))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let pane = Block::default()
        .borders(Borders::ALL)
        .border_style(content_block_border_style)
        .title(format!(" {} ", state.current_tool.title()))
        .title_alignment(Alignment::Center);

    let inner = pane.inner(area);

    frame.render_widget(pane, area);

    state.current_tool.render(
        frame,
        inner,
        config_data,
        &mut state.service_status,
        &mut state.token_generator,
        &mut state.jira,
    );
}
