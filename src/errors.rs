use std::env;
use std::error;
use std::io;
use std::fmt;

#[derive(Debug)]
pub enum WindowsSpotlightError {
    EnvVar(env::VarError),
    Io(io::Error),
}

pub type WindowsSpotlightResult<T> = Result<T, WindowsSpotlightError>;

impl fmt::Display for WindowsSpotlightError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WindowsSpotlightError::EnvVar(ref error) => {
                write!(formatter, "Environment variable error: {}", error)
            }
            WindowsSpotlightError::Io(ref error) => write!(formatter, "IO error: {}", error),
        }
    }
}

impl error::Error for WindowsSpotlightError {
    fn description(&self) -> &str {
        match *self {
            WindowsSpotlightError::EnvVar(ref error) => error.description(),
            WindowsSpotlightError::Io(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            WindowsSpotlightError::EnvVar(ref error) => Some(error),
            WindowsSpotlightError::Io(ref error) => Some(error),
        }
    }
}

impl From<env::VarError> for WindowsSpotlightError {
    fn from(error: env::VarError) -> WindowsSpotlightError {
        WindowsSpotlightError::EnvVar(error)
    }
}

impl From<io::Error> for WindowsSpotlightError {
    fn from(error: io::Error) -> WindowsSpotlightError {
        WindowsSpotlightError::Io(error)
    }
}