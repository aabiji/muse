use serde::Deserialize;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::process::Command;

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

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new() -> Self {
        Server {
            config: load_config(),
        }
    }

    fn handle_client(&mut self, mut stream: TcpStream) {
        let mut buffer: [u8; crate::MSG_SIZE] = [0; crate::MSG_SIZE];
        let amount_read = stream.read(&mut buffer).unwrap();
        if amount_read != crate::MSG_SIZE {
            // houston we have a problem
        }

        let msg = String::from_utf8(buffer.to_vec()).unwrap();
        println!("{}", msg);

        let response = "Hello! This is a test response!";
        stream.write(response.as_bytes()).unwrap();
    }

    pub fn start(&mut self) {
        println!("Starting server ...");
        let listener = TcpListener::bind(crate::SERVER_ADDR).unwrap();
        for stream in listener.incoming() {
            self.handle_client(stream.unwrap());
        }
    }
}

pub fn spawn_if_not_spawned() {
    if let Ok(_) = TcpStream::connect(crate::SERVER_ADDR) {
        return; // Server is already spawned
    }

    // TODO: replace this path with an actual command
    let path = "/home/aabiji/dev/muse/target/debug/muse";
    Command::new(path).arg("--server-mode").spawn();

    // Wait to process to start
    std::thread::sleep(std::time::Duration::from_secs(1));
}
