use std::path::{Path, PathBuf};
use std::time::Duration;

pub enum LogType {
    Info,
    Warning,
    Error,
}

pub fn log(msg: String, log_type: LogType) {
    let ansi_escape_codes = match log_type {
        LogType::Error => "\x1b[1;31m",
        LogType::Warning => "\x1b[1;33m",
        _ => "\x1b[1;37m",
    };
    println!("{ansi_escape_codes}{msg}");
}

pub fn format_time(d: Duration) -> String {
    let total = d.as_secs();
    let seconds = total % 60;
    let minutes = (total / 60) % 60;
    let hours = (total / 60 / 60) % 60;

    let mut time = String::new();
    if hours > 0 {
        time.push_str(&format!("{hours}h "));
    }
    if minutes > 0 {
        time.push_str(&format!("{minutes}m "));
    }
    if seconds > 0 {
        time.push_str(&format!("{seconds}s"));
    }
    time
}

pub fn home_path(entry: &str) -> String {
    let home_directory = home::home_dir().unwrap();
    let mut base = PathBuf::from(home_directory);
    base.push(entry);
    base.to_str().unwrap().to_string()
}

pub fn is_supported_codec(file: &Path) -> bool {
    let supported = ["mp3", "mp4", "wav", "ogg", "flac"];
    let extension = file.extension().unwrap().to_str().unwrap();
    if !supported.contains(&extension) {
        return false;
    }
    true
}
