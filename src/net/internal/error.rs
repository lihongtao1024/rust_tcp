use crate::Event;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Fatal(String),
    Io(std::io::Error),
    Event(tokio::sync::mpsc::error::SendError<Event>),
    Task(tokio::task::JoinError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            Error::Fatal(err) => err.clone(),
            Error::Io(err) => err.to_string(),
            Error::Event(err) => err.to_string(),
            Error::Task(err) => err.to_string(),
        };

        write!(f, "{}", err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::Task(err)
    }
}

impl From<tokio::sync::mpsc::error::SendError<Event>> for Error {
    fn from(err: tokio::sync::mpsc::error::SendError<Event>) -> Self {
        Error::Event(err)
    }
}