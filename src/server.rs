use std::io::prelude::*;
use std::process::Command;
use serde::{Serialize, Deserialize};
use std::net::{TcpListener, TcpStream};

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

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Start, // Start audio playback
    Stop,  // Stop audio playback
}

#[derive(Serialize, Deserialize, Debug)]
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

    pub fn run(&mut self) {
        let listener = TcpListener::bind(ADDR).unwrap();
        for conn in listener.incoming() {
            self.handle_client_connection(conn.unwrap());
        }
    }

    fn start_playback(&self) -> Response {
        Response::Success("Start playing music".to_string())
    }

    fn stop_playback(&self) -> Response {
        Response::Success("Stop playing music".to_string())
    }

    fn handle_client_connection(&mut self, mut conn: TcpStream) {
        println!("Running the server ... READING");
        let mut buffer: Vec<u8> = Vec::new();
        let read = conn.read(&mut buffer).unwrap();
        println!("Running the server ... READ {}", read);

        let buffer = crate::read_data(&mut conn);
        let request: Request = serde_json::from_slice(&buffer).unwrap();

        let response = match request {
            Request::Start => self.start_playback(),
            Request::Stop => self.stop_playback()
        };

        let mut data: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut data, &response).unwrap();
        crate::write_data(&mut conn, data);
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
