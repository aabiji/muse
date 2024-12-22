mod audio;
mod config;
mod ipc;
mod util;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: ipc::Command,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        ipc::Command::Start => {
            let mut server = ipc::Server::new();
            server.run();
        }
        arg => {
            let mut client = ipc::Client {};
            client.run(arg);
        }
    };
}
