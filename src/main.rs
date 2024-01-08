use std::io::prelude::*;
use std::net::TcpStream;
use colored::Colorize;

mod server;

fn print_help() {
    println!("{}", r#"
muse is a cli program to play background music.

Usage:
muse [Options]

Options:
start        Start playing music.
stop         Stop playing music.
info         Show info about currently played audio. 
    "#);
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
    let mut data: Vec<u8> = Vec::new();
    data.resize(1, 0);
    stream.read_exact(&mut data).unwrap();
    let length = data[0] as usize;

    data.clear();
    data.resize(length, 0);
    stream.read_exact(&mut data).unwrap();

    data
}

fn send_request(r: server::Request) -> server::Response {
    println!("Running the client ...");
    let mut conn = TcpStream::connect(server::ADDR).unwrap();
    println!("Running the client ... CONNECTED");

    let mut data: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut data, &r).unwrap();
    write_data(&mut conn, data);

    let buffer = read_data(&mut conn);
    let response: server::Response = serde_json::from_slice(&buffer).unwrap();

    conn.shutdown(std::net::Shutdown::Both).unwrap();
    response
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let max_args = 2;
    if args.len() != max_args {
        print_help();
        return;
    }

    if args.contains(&server::SERVER_MODE_FLAG.to_string()) {
        let mut server = server::Server::new();
        server.run();
        return;
    }

    if !args.contains(&"start".to_string()) && !args.contains(&"stop".to_string()) {
        print_help();
        return;
    }

    server::spawn_if_not_spawned();

    let request = if args[1] == "start" {
        server::Request::Start
    } else {
        server::Request::Stop
    };

    match send_request(request) {
        server::Response::Success(msg) => println!("{}", msg.green()),
        server::Response::Error(msg) => println!("{}", msg.red()),
    };
}
