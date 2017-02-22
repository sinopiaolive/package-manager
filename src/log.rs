use std::fmt::{self, Display};

#[derive(PartialEq, PartialOrd, Debug, RustcDecodable)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
        }
    }
}

pub trait Log {
    fn log(&self, level: LogLevel, message: &Display);

    fn error(&self, message: &Display) {
        self.log(LogLevel::Error, message)
    }

    fn warning(&self, message: &Display) {
        self.log(LogLevel::Warning, message)
    }

    fn info(&self, message: &Display) {
        self.log(LogLevel::Info, message)
    }

    fn debug(&self, message: &Display) {
        self.log(LogLevel::Debug, message)
    }
}

pub struct StdLogger {
    log_level: LogLevel
}

impl StdLogger {
    pub fn new(level: LogLevel) -> StdLogger {
        StdLogger {log_level: level}
    }
}

impl Log for StdLogger {
    fn log(&self, level: LogLevel, message: &Display) {
        if level <= self.log_level {
            println!("[{}] {}", level, message)
        }
    }
}
