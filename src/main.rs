use std::io::prelude::*;
use std::net::TcpStream;

mod server;

const MSG_SIZE: usize = 256;
const SERVER_ADDR: &str = "127.0.0.1:1234";

fn main() {
    let mut server = server::Server::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--server-mode" {
        server.start();
        return;
    }

    server::spawn_if_not_spawned();

    let msg = "Hello! This is a test request!";
    let mut conn = TcpStream::connect(SERVER_ADDR).unwrap();
    conn.write(msg.as_bytes()).unwrap();

    let mut response: [u8; MSG_SIZE] = [0; MSG_SIZE];
    conn.read(&mut response).unwrap();

    let msg = String::from_utf8(response.to_vec()).unwrap();
    println!("{}", msg);
}
