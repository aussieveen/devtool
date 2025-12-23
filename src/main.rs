mod app;
mod ui;
mod state;
mod events;
mod config;

use crate::app::App;
use crate::config::Config;
use serde_yaml;

const CONFIG_FILE: &str = "config/config.yaml";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let config: Config = load_config();
    let result = App::new(config).run(terminal).await;
    ratatui::restore();
    result
}

fn load_config() -> Config {
    let f = std::fs::read_to_string(CONFIG_FILE);
    serde_yaml::from_str(f.unwrap().as_str()).unwrap()
}

