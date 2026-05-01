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
use crate::config::loader::ConfigLoader;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let config_loader = ConfigLoader::new(".devtool", "config.yaml");
    let config = config_loader.read_or_create_config()?;
    let result = App::new(config, config_loader).run(terminal).await;
    ratatui::restore();
    result
}
