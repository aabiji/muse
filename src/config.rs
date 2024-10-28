use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::{Path, PathBuf};

const AUDIO_FOLDER: &str = "Music";
const CONFIG_FILE: &str = ".muse.conf";

fn home_path(entry: &str) -> String {
    let home_directory = home::home_dir().unwrap();
    let mut base = PathBuf::from(home_directory);
    base.push(entry);
    base.to_str().unwrap().to_string()
}

#[derive(Serialize, Deserialize)]
pub enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub start_point: u64,
    pub resume_playback: bool,
    pub playback_order: PlaybackOrder,
    pub audio_directories: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            start_point: 0,
            resume_playback: true,
            playback_order: PlaybackOrder::Random,
            audio_directories: vec![home_path(AUDIO_FOLDER)],
        }
    }
}


pub fn save(config: &Config) {
    let serialized = toml::to_string(config).unwrap();
    std::fs::write(home_path(CONFIG_FILE), serialized).unwrap();
}

pub fn load() -> Result<Config, Box<dyn Error>> {
    // Create the config file if it doesn't already exist
    let path = home_path(CONFIG_FILE);
    if !Path::new(&path).exists() {
        let default = Config::new();
        save(&default);
        return Ok(default);
    }

    let file = std::fs::read_to_string(path)?;
    let mut config: Config = toml::from_str(&file)?;

    if !config.resume_playback {
        config.start_point = 0;
    }

    if config.audio_directories.is_empty() {
        // Fallback to the default
        config.audio_directories = vec![home_path(AUDIO_FOLDER)];
    }

    for path in &config.audio_directories {
        if !Path::new(&path).exists() {
            let msg = format!("{} not found.", path);
            return Err(Box::<dyn Error>::from(msg));
        }
    }

    Ok(config)
}