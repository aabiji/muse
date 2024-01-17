mod audio;
mod config;
mod net;

fn print_help() {
    println!(
        "{}",
        r#"
Muse is a minimal cli music player.

Usage:
muse [options]

Options:
play         Start playing music.
pause        Pause the playing music.
stop         Stop the audio playback server.
    "#
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        print_help();
        return;
    }

    let arg = &args[1];
    if arg == net::SERVER_MODE_FLAG {
        let mut server = net::Server::new();
        server.run();
        return;
    }

    let possible_user_args = ["play", "pause", "stop"];
    if !possible_user_args.contains(&arg.as_str()) {
        print_help();
        return;
    }

    let mut client = net::Client {};
    client.run(arg);
}
