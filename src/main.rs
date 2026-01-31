mod app;
mod config;
mod environment;
mod events;
mod persistence;
mod state;
mod ui;
mod client;

use crate::app::App;
use crate::config::Config;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let config: Config = config::read_config();
    let result = App::new(config).run(terminal).await;
    ratatui::restore();
    result
}
