mod app;
mod client;
mod config;
mod environment;
mod error;
mod events;
mod input;
mod persistence;
mod state;
mod ui;
mod utils;

use crate::app::App;
use crate::config::loader::ConfigLoader;
use crate::config::model::Config;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let config: Config =
        ConfigLoader::new(".devtool", "config.yaml").read_config();
    let result = App::new(config).run(terminal).await;
    ratatui::restore();
    result
}
