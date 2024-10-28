use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{exit, Command};

use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::util;
use crate::audio::Playback;

pub const ADDR: &str = "127.0.0.1:1234";

#[derive(PartialEq, Subcommand, Serialize, Deserialize)]
pub enum Request {
    /// Play background music.
    Play,
    /// Pause background music.
    Pause,
    /// Start the playback server.
    /// The process will hang until
    /// the server is shutdown.
    Start,
    /// Stop the playback server.
    Stop,
}

#[derive(Serialize, Deserialize)]
enum Response {
    Success(String),
    Error(String),
}

fn write_data(stream: &mut TcpStream, data: Vec<u8>) {
    // Custom wire format used to transfer data
    // between the client and server: [ LENGTH, DATA ]
    stream.write(&[data.len() as u8]).unwrap();
    stream.write_all(&data).unwrap();
    stream.flush().unwrap();
}

// TODO: Is it worth it to implement a read_to_end function?
fn read_data(stream: &mut TcpStream) -> Vec<u8> {
    // Custom wire format used to transfer data
    // between the client and server: [ LENGTH, DATA ]
    let mut data = vec![0];
    stream.read_exact(data.as_mut_slice()).unwrap();
    let length = data[0] as usize;

    data.clear();
    data.resize(length, 0);
    stream.read_exact(data.as_mut_slice()).unwrap();

    data
}

// Responsible for receiving client commands and
// executing them, returning a response.
pub struct Server {
    playback: Playback,
}

impl Server {
    pub fn new() -> Self {
        Self {
            playback: Playback::new(),
        }
    }

    fn handle_request(&mut self, mut stream: TcpStream) -> bool {
        let mut shutdown = false;

        let mut buffer: Vec<u8> = Vec::new();
        stream.read(&mut buffer).unwrap(); // TODO: why are we calling this???

        let buffer = read_data(&mut stream);
        let request: Request = serde_json::from_slice(&buffer).unwrap();

        let result = match request {
            Request::Play => self.playback.play(),
            Request::Pause => self.playback.pause(),
            Request::Stop => {
                shutdown = true;
                self.playback.stop(true)
            }
            _ => Ok(String::new()),
        };

        let response = match result {
            Ok(msg) => Response::Success(msg),
            Err(msg) => Response::Error(msg),
        };

        let mut data: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut data, &response).unwrap();
        write_data(&mut stream, data);

        if let Response::Error(_) = response {
            self.playback.stop(false).unwrap();
            exit(1);
        }

        shutdown
    }

    pub fn run(&mut self) {
        if Server::is_running() {
            util::log("Audio server is already running".to_string(), util::LogType::Error);
            exit(1);
        }

        let listener = TcpListener::bind(ADDR).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            if self.handle_request(stream) {
                break;
            }
        }
    }

    fn is_running() -> bool {
        if let Err(_) = TcpListener::bind(ADDR) {
            return true;
        }
        false
    }

    fn spawn_background_process() {
        if Server::is_running() {
            return;
        }

        let exe_path = std::env::current_exe().unwrap();
        let path = exe_path.to_str().unwrap();
        Command::new(path).arg("start").spawn().unwrap();

        // Wait for the server process to start and initialize.
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// Responsible for sending requests to the
// server and outputting the server's response.
pub struct Client;

impl Client {
    fn send_request(&self, r: Request) -> Response {
        let mut stream = TcpStream::connect(ADDR).unwrap();

        let mut data: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut data, &r).unwrap();
        write_data(&mut stream, data);

        let buffer = read_data(&mut stream);
        let response: Response = serde_json::from_slice(&buffer).unwrap();

        stream.shutdown(Shutdown::Both).unwrap();
        response
    }

    pub fn run(&mut self, request: Request) {
        if !Server::is_running() && request == Request::Stop {
            util::log("No audio server is running".to_string(), util::LogType::Error);
            return;
        }

        Server::spawn_background_process();

        match self.send_request(request) {
            Response::Success(msg) => util::log(msg, util::LogType::Info),
            Response::Error(msg) => util::log(msg, util::LogType::Error),
        };
    }
}
