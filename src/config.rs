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

    // FIXME: Move this method out of the struct
    //        refer to path field in the struct
    fn path() -> String {
        // Default config path: ~/.muse.conf
        // FIXME: why are we doing this: converting from a pathbuf to a string, to a pathbuf again???
        // TODO; can we replace the home dependency???
        let dir = home::home_dir().unwrap();
        let path_str = dir.display().to_string();
        let mut path = PathBuf::from(&path_str);
        path.push(".muse.conf");
        String::from(path.to_str().unwrap())
    }

    // Move this outside the struct
    // Instead of changing the values in place, returned the parsed struct
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO: create the config file if it doesn't already exist.
        //       create a default list of audio folders
        let path = Config::path();
        if !Path::new(&path).exists() {
            self.save();
            return Ok(());
        }

        // FIXME: refer to 'path' above. bubble up error if file doesn't exist.
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

    // Move this outside the struct
    pub fn save(&self) {
        let serialized = toml::to_string(&self).unwrap();
        std::fs::write(Config::path(), serialized).unwrap();
    }
}