mod audio;
mod config;
mod net;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: net::Request,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        net::Request::Start => {
            let mut server = net::Server::new();
            server.run();
        }
        arg => {
            let mut client = net::Client {};
            client.run(arg);
        }
    };
}
