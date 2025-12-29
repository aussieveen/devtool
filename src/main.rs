mod app;
mod config;
mod environment;
mod events;
mod state;
mod ui;

use crate::app::App;
use crate::config::Config;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE: &str = ".devtool/config.yaml";

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
    let home_dir = dirs::home_dir().expect("Could not find home directory");

    // Append your file path
    let file_path: PathBuf = home_dir.join(CONFIG_FILE);

    // Read the file
    let config = fs::read_to_string(file_path).unwrap();

    serde_yaml::from_str(config.as_str()).unwrap()
}
