use std::{
    error,
    fmt::{self, Display, Formatter},
    result,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Client,
    Server,
    NotFound,
    Unauthenticated,
    Other,
}

#[derive(Debug)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { message, kind }
    }

    pub fn other(message: String) -> Self {
        Self::new(ErrorKind::Other, message)
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            ErrorKind::Unauthenticated => "authentication required",
            _ => &self.message,
        };

        f.write_str(message)
    }
}

impl error::Error for Error {}

pub type Result<T> = result::Result<T, Error>;
