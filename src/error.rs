use core::num::ParseIntError;
use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as ioError;

#[derive(Debug)]
pub enum AppError {
    Error(String),

    Io(ioError),

    InvalidInt(ParseIntError),

    Yaml(serde_yaml::Error),
}

impl Error for AppError {}

impl From<String> for AppError {
    fn from(s: String) -> AppError {
        AppError::Error(s)
    }
}

impl From<ioError> for AppError {
    fn from(value: ioError) -> AppError {
        AppError::Io(value)
    }
}

impl From<ParseIntError> for AppError {
    fn from(value: ParseIntError) -> AppError {
        AppError::InvalidInt(value)
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(value: serde_yaml::Error) -> AppError {
        AppError::Yaml(value)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            AppError::Error(s) => write!(f, "Encountered error: {}", s),
            AppError::Io(err) => write!(f, "{}", err),
            AppError::InvalidInt(err) => write!(f, "{}", err),
            AppError::Yaml(err) => write!(f, "{}", err),
        }
    }
}
