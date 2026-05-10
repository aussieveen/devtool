extern crate core;

mod app;
mod client;
mod config;
mod environment;
mod error;
mod event;
mod input;
mod persistence;
pub mod popup;
mod state;
mod ui;
mod utils;

use crate::app::App;
use crate::config::loader::ConfigFile;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let config_file = ConfigFile::new(".devtool", "config.yaml");
    let config = config_file.read_or_create_config()?;
    let result = App::new(config, config_file).run(terminal).await;
    ratatui::restore();
    result
}
