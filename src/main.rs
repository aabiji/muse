use clap::Parser;

mod audio;
mod config;
mod net;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: net::Request,

    server_flag: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(flag) = cli.server_flag {
        if flag == net::SERVER_MODE_FLAG {
            let mut server = net::Server::new();
            server.run();
            return;
        }
    }

    let mut client = net::Client {};
    client.run(cli.command);
}
