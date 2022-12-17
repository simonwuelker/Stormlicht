mod macros;

use std::fmt;
use std::io::{stdout, Write};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub struct LogMessage {
    /// The file name and line number where the log message was created
    pub source: (&'static str, u32),
    pub level: Level,
    pub target: Option<&'static str>,
    pub text: String,
}

pub fn log_generic(message: LogMessage) {
    let LogMessage {
        source,
        level,
        target,
        text,
    } = message;
    let mut f = stdout().lock();
    if let Some(target) = target {
        writeln!(f, "[{}:{} {level} {target}]: {text}", source.0, source.1).unwrap();
    } else {
        writeln!(f, "[{}:{} {level}]: {text}", source.0, source.1).unwrap();
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warn => write!(f, "WARN"),
            Self::Info => write!(f, "INFO"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Trace => write!(f, "TRACE"),
        }
    }
}
