// Std lib
use std::error::Error as StdError;
use std::fmt;

// Third party
use envy;
use rusoto_ssm::GetParametersByPathError;

/// Represents possible errors
#[derive(Debug)]
pub enum Error {
    /// Returned when parameter store request fails
    Store(GetParametersByPathError),
    /// Returned with deserialization fails
    Envy(envy::Error),
}

impl From<GetParametersByPathError> for Error {
    fn from(err: GetParametersByPathError) -> Self {
        Error::Store(err)
    }
}

impl From<envy::Error> for Error {
    fn from(err: envy::Error) -> Self {
        Error::Envy(err)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::Store(e) => e.description(),
            Error::Envy(e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match self {
            Error::Store(e) => e.cause(),
            Error::Envy(e) => e.cause(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Error::Store(e) => write!(fmt, "{}", e),
            Error::Envy(e) => write!(fmt, "{}", e),
        }
    }
}
