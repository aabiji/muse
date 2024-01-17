use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub audio_folder_path: String,
    pub playback_order: PlaybackOrder,
    pub resume_playback: bool,
    pub start_point: u64,
}

impl Config {
    pub fn default() -> Self {
        Config {
            audio_folder_path: String::new(),
            playback_order: PlaybackOrder::Random,
            resume_playback: true,
            start_point: 0,
        }
    }

    fn path() -> String {
        // Default config path: ~/.muse.conf
        let dir = home::home_dir().unwrap();
        let path_str = dir.display().to_string();
        let mut path = PathBuf::from(&path_str);
        path.push(".muse.conf");
        String::from(path.to_str().unwrap())
    }

    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let path = Config::path();
        if !Path::new(&path).exists() {
            self.save();
            return Ok(());
        }

        let file = std::fs::read_to_string(Config::path()).unwrap();
        let mut config: Config = toml::from_str(&file)?;

        if !config.resume_playback {
            config.start_point = 0;
        }

        if !Path::new(&config.audio_folder_path).exists() {
            let msg = format!(
                "Path to audio folder ({}) not found.",
                config.audio_folder_path
            );
            return Err(Box::<dyn Error>::from(msg));
        }

        *self = config;
        Ok(())
    }

    pub fn save(&self) {
        let serialized = toml::to_string(&self).unwrap();
        std::fs::write(Config::path(), serialized).unwrap();
    }
}
