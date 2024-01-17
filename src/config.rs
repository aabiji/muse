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
    pub start_point: u64,
    pub resume_playback: bool,
    pub playback_order: PlaybackOrder,
    pub audio_directories: Vec<String>,
}

impl Config {
    pub fn default() -> Self {
        Config {
            start_point: 0,
            resume_playback: true,
            audio_directories: Vec::new(),
            playback_order: PlaybackOrder::Random,
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

        for path in &config.audio_directories {
            if !Path::new(&path).exists() {
                let msg = format!("Path to audio folder ({}) not found.", path);
                return Err(Box::<dyn Error>::from(msg));
            }
        }

        *self = config;
        Ok(())
    }

    pub fn save(&self) {
        let serialized = toml::to_string(&self).unwrap();
        std::fs::write(Config::path(), serialized).unwrap();
    }
}
