mod client;
mod server;

use serde::Deserialize;

#[derive(Deserialize)]
enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Deserialize)]
struct Config {
    audio_folder_path: String,
    continue_playback: bool,
    playback_order: PlaybackOrder,
}

fn load_config() -> Config {
    let path = "config.toml";
    let file = std::fs::read_to_string(path).unwrap();
    let config: Config = toml::from_str(&file).unwrap();
    config
}

fn main() {
    let mut client = client::Client{};
    let mut server = server::Server{};
    let config = load_config();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--server-mode" {
        server.start(&config); 
    } else {
        client.start(&config);
    }
}
