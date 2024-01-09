use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::Command;

use colored::Colorize;
use serde::{Deserialize, Serialize};

pub const ADDR: &str = "127.0.0.1:1234";
pub const SERVER_MODE_FLAG: &str = "--server-mode";

#[derive(Serialize, Deserialize)]
enum Request {
    Start, // Start audio playback
    Stop,  // Stop audio playback
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

fn send_request(r: Request) -> Response {
    let mut stream = TcpStream::connect(ADDR).unwrap();

    let mut data: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut data, &r).unwrap();
    write_data(&mut stream, data);

    let buffer = read_data(&mut stream);
    let response: Response = serde_json::from_slice(&buffer).unwrap();

    stream.shutdown(Shutdown::Both).unwrap();
    response
}

pub fn run_client(arg: &str) {
    spawn_server();

    let request = if arg == "start" {
        Request::Start
    } else {
        Request::Stop
    };
    match send_request(request) {
        Response::Success(msg) => println!("{}", msg.green()),
        Response::Error(msg) => println!("{}", msg.red()),
    };
}

fn send_response(mut stream: TcpStream) {
    let mut buffer: Vec<u8> = Vec::new();
    stream.read(&mut buffer).unwrap();

    let buffer = read_data(&mut stream);
    let request: Request = serde_json::from_slice(&buffer).unwrap();

    let response = match request {
        Request::Start => crate::audio::start_playback(),
        Request::Stop => crate::audio::stop_playback(),
    };

    let mut data: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut data, &response).unwrap();
    write_data(&mut stream, data);
}

pub fn run_server() {
    let listener = TcpListener::bind(ADDR).unwrap();
    for stream in listener.incoming() {
        send_response(stream.unwrap());
    }
}

pub fn spawn_server() {
    // The server process is already running
    if let Err(_) = TcpListener::bind(ADDR) {
        return;
    }

    let exe_path = std::env::current_exe().unwrap();
    let path = exe_path.to_str().unwrap();
    Command::new(path).arg(SERVER_MODE_FLAG).spawn().unwrap();

    // Wait to process to start
    std::thread::sleep(std::time::Duration::from_secs(1));
}
