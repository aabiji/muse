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