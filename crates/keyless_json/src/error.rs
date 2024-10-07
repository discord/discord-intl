use serde::ser;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("IOError: {0}")]
    IoError(std::io::Error),
    #[error("{0}")]
    CustomError(String),
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}

impl ser::Error for Error {
    #[cold]
    fn custom<T: std::fmt::Display>(msg: T) -> Error {
        Error::CustomError(msg.to_string())
    }
}
