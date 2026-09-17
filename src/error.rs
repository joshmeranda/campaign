use std::error::Error;
use std::num::ParseIntError;
use std::fmt;

#[derive(Debug)]
pub struct AppError {
	s: String,
}

impl Error for AppError {}

impl From<String> for AppError {
	fn from(s: String) -> AppError {
		AppError {
			s: s,
		 }
	}
}

impl From<std::io::Error> for AppError{
	fn from(value: std::io::Error) -> AppError {
		AppError{ s:
			value.to_string()
		}
	}
}

impl From<ParseIntError> for AppError {
	fn from(value: ParseIntError) -> AppError {
		AppError{ s:
			value.to_string()
		}
	}
}

impl From<serde_yaml::Error> for AppError{
	fn from(value: serde_yaml::Error) -> AppError {
		AppError{ s:
			value.to_string()
		}
	}
}

impl fmt::Display for AppError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.s)
	}
}
