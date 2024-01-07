use std::io::prelude::*;
use std::process::Command;
use serde::{Serialize, Deserialize};
use std::net::{TcpListener, TcpStream};

pub const MSG_SIZE: usize = 256;
pub const ADDR: &str = "127.0.0.1:1234";
pub const SERVER_MODE_FLAG: &str = "--server-mode";

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

#[derive(Serialize, Deserialize)]
pub enum Request {
    Start, // Start audio playback
    Stop,  // Stop audio playback
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Success(String),
    Error(String),
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

    pub fn start(&mut self) {
        let listener = TcpListener::bind(ADDR).unwrap();
        for stream in listener.incoming() {
            self.exec_client_request(stream.unwrap());
        }
    }

    fn start_playback(&self) -> Response {
        Response::Success("Start playing music".to_string())
    }

    fn stop_playback(&self) -> Response {
        Response::Success("Stop playing music".to_string())
    }

    fn exec_client_request(&mut self, mut stream: TcpStream) {
        let mut buffer: [u8; MSG_SIZE] = [0; MSG_SIZE];
        let bytes_read = stream.read(&mut buffer).unwrap();
        println!("{}", bytes_read); 

        let slice = &buffer[0..bytes_read];
        let request: Request = serde_json::from_slice(slice).unwrap();
        let response = match request {
            Request::Start => self.start_playback(),
            Request::Stop => self.stop_playback()
        };

        serde_json::to_writer(stream, &response).unwrap();
   }
}

pub fn spawn_if_not_spawned() {
    if let Ok(_) = TcpStream::connect(ADDR) {
        return; // Server is already spawned
    }

    // TODO: replace this path with an actual command
    let path = "/home/aabiji/dev/muse/target/debug/muse";
    Command::new(path).arg(SERVER_MODE_FLAG).spawn().unwrap();

    // Wait to process to start
    std::thread::sleep(std::time::Duration::from_secs(1));
}
