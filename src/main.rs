mod audio;
mod config;
mod ipc;
mod util;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: ipc::Request,
    // TODO: add a 'add URL' sub command
    // TODO: add a 'uninstall' sub command
}

/*
Muse v2 checklist:
- Install script
- Bluetooth earbud pairing (pausing when disconnected?)
- Refactor the codebase (make sure to replace all unwraps with proper error propagation)
- TODO: rewrite the readme
*/

fn main() {
    let cli = Cli::parse();
    match cli.command {
        ipc::Request::Start => {
            let mut server = ipc::Server::new();
            server.run();
        }
        arg => {
            let mut client = ipc::Client {};
            client.run(arg);
        }
    };
}
