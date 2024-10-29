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
}

/*
Muse v2 checklist:
- Install script
- Pause playback when bluetooth earbuds are disconnected
- Refactor the codebase (make sure to replace all major unwraps with proper error propagation)
- 'add URL' sub command
- 'uninstall' sub command
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
