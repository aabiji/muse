mod audio;
mod net;

fn print_help() {
    println!(
        "{}",
        r#"
muse is a cli program to play background music.

Usage:
muse [options]

Options:
start        Start playing music.
stop         Stop playing music.
info         Show info about currently played audio.
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
        net::run_server();
        return;
    }

    let possible_user_args = ["start", "stop", "info"];
    if !possible_user_args.contains(&arg.as_str()) {
        print_help();
        return;
    }

    net::run_client(arg);
}
