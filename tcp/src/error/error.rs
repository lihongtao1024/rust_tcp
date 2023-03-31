use std::fmt::Display;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    Module(&'static str),
    System(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Module(s) => write!(f, "Error: {{ Module: {} }}", s),
            Error::System(s) => write!(f, "Error: {{ System: {} }}", s),
        }
    }
}

impl StdError for Error {

}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        let s = Box::new(value.to_string());
        Error::Module(Box::leak(s))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        let s = Box::new(value.to_string());
        Error::Module(Box::leak(s))
    }
}