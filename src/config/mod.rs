pub mod types;

use anyhow::Result;
use std::fs;
use types::{Config, config_path};

pub fn load() -> Config {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: invalid config at {}: {}, using defaults", path.display(), e);
                Config::default()
            }
        },
        Err(_) => Config::default(),
    }
}

pub fn load_from_str(content: &str) -> Result<Config> {
    Ok(toml::from_str(content)?)
}

pub fn save(cfg: &Config) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match toml::to_string_pretty(cfg) {
        Ok(content) => {
            if let Err(e) = fs::write(&path, content) {
                eprintln!("Warning: failed to save config to {}: {}", path.display(), e);
            }
        }
        Err(e) => {
            eprintln!("Warning: failed to serialize config: {}", e);
        }
    }
}
