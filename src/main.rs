use std::io::prelude::*;
use std::net::TcpStream;
use colored::Colorize;

mod server;

fn print_help() {
    println!("{}", format!(r#"
muse is a cli program to play background music.

Usage:
muse [Options]

Options:
start             Start playing music.
stop              Stop playing music.
{}     Start the audio playback server.
    "#, server::SERVER_MODE_FLAG));
}

fn send_request(c: server::Request) -> server::Response {
    let mut conn = TcpStream::connect(server::ADDR).unwrap();
    serde_json::to_writer(&conn, &c).unwrap();

    let mut buffer: [u8; server::MSG_SIZE] = [0; server::MSG_SIZE];
    let bytes_read = conn.read(&mut buffer).unwrap();

    let slice = &buffer[0..bytes_read];
    let response: server::Response = serde_json::from_slice(slice).unwrap();
    response
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&server::SERVER_MODE_FLAG.to_string()) {
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
        server::Request::Start
    } else {
        server::Request::Stop
    };

    match send_request(command) {
        server::Response::Success(msg) => println!("{:?}", msg.green()),
        server::Response::Error(msg) => println!("{:?}", msg.red()),
    };
}
