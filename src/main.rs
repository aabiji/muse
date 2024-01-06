use std::io::prelude::*;
use std::net::TcpStream;
use serde::{Serialize, Deserialize};

mod server;

const MSG_SIZE: usize = 256;
const SERVER_ADDR: &str = "127.0.0.1:1234";

#[derive(Serialize, Deserialize)]
enum ClientCommand {
    Start, // Start audio playback
    Stop,  // Stop audio playback
}

#[derive(Serialize, Deserialize)]
enum ServerResponse {
    Success(String),
    Error(String),
}

fn send_command(c: ClientCommand) -> ServerResponse {
    let mut conn = TcpStream::connect(SERVER_ADDR).unwrap();
    let msg = serde_json::to_string(&c).unwrap();
    conn.write(msg.as_bytes()).unwrap();

    let mut buffer: [u8; MSG_SIZE] = [0; MSG_SIZE];
    conn.read(&mut buffer).unwrap();

    let resp: ServerResponse = serde_json::from_slice(&buffer).unwrap();
    resp
}

fn print_help() {
    println!("{}", r#"
muse is a cli program to play background music.

Usage:
muse [Options]

Options:
start             Start playing music.
stop              Stop playing music.
--server-mode     Start the audio playback server.
    "#);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--server-mode".to_string()) {
        let mut server = server::Server::new();
        server.start();
        return;
    }

    if !args.contains(&"start".to_string()) && !args.contains(&"stop".to_string()) {
        print_help();
        return;
    }

    server::spawn_if_not_spawned();

    let command = if args[1] == "start" {
        ClientCommand::Start
    } else {
        ClientCommand::Stop
    };

    match send_command(command) {
        ServerResponse::Success(msg) => println!("Sucess! {}", msg),
        ServerResponse::Error(msg) => println!("Error! {}", msg),
    };
}
