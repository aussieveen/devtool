mod app;
mod config;
mod environment;
mod events;
mod state;
mod ui;
mod persistence;

use crate::app::App;
use crate::config::Config;

const CONFIG_FILE: &str = ".devtool/config.yaml";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let config: Config = config::read_config();
    let result = App::new(config).run(terminal).await;
    ratatui::restore();
    result
}
