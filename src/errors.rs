use std::{error, fmt, io, num, result, sync::mpsc};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Battery(battery::Error),
    Io(io::Error),
    Channel(mpsc::RecvError),
    Crossterm(String),
    Database(rusqlite::Error),
    Daemonize(String),
    DaemonAlreadyRunning,
    DaemonNotRunning,
    InvalidPid(num::ParseIntError),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Battery(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::Channel(e) => Some(e),
            Error::Database(e) => Some(e),
            Error::InvalidPid(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Crossterm(msg) => write!(f, "Crossterm error: {}", msg),
            Error::Daemonize(msg) => write!(f, "Daemonize error: {}", msg),
            Error::DaemonAlreadyRunning => f.write_str("Daemon is already running"),
            Error::DaemonNotRunning => f.write_str("Daemon is not running"),
            Error::Battery(e) => fmt::Display::fmt(e, f),
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::Channel(e) => fmt::Display::fmt(e, f),
            Error::Database(e) => fmt::Display::fmt(e, f),
            Error::InvalidPid(e) => write!(f, "Invalid PID: {}", e),
        }
    }
}

impl From<battery::Error> for Error {
    fn from(e: battery::Error) -> Self {
        Error::Battery(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(e: mpsc::RecvError) -> Self {
        Error::Channel(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Database(e)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Self {
        Error::InvalidPid(e)
    }
}
