
#[derive(Deserialize)]
pub enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Deserialize)]
pub struct Config {
    audio_folder_path: String,
    continue_playback: bool,
    playback_order: PlaybackOrder,
}

pub fn load_config() -> Config {
    let path = "config.toml";
    let file = std::fs::read_to_string(path).unwrap();
    let config: Config = toml::from_str(&file).unwrap();
    config
}
