use std::{
    error,
    fmt::{self, Display},
    process::ExitCode,
    result,
};

#[derive(Debug)]
pub enum Error {
    Config(String),
    Client(minty::Error),
    Other(String),
}

impl Error {
    pub fn report(&self) -> ExitCode {
        match self {
            Self::Config(_) => ExitCode::from(78),
            _ => ExitCode::FAILURE,
        }
    }
}

impl From<minty::Error> for Error {
    fn from(value: minty::Error) -> Self {
        Self::Client(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => f.write_str(err),
            Self::Client(err) => err.fmt(f),
            Self::Other(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {}

pub type Result<T> = result::Result<T, Error>;
