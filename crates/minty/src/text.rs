#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::{
    error::Error as StdError,
    fmt::{self, Display},
    result,
    str::FromStr,
};

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    ContainsNewlines,
    Empty,
}

#[derive(Clone, Copy, Debug)]
pub struct Error {
    pub context: &'static str,
    pub kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        match self.kind {
            ContainsNewlines => {
                write!(f, "{} must not contain newlines", self.context)
            }
            Empty => write!(f, "{} must not be empty", self.context),
        }
    }
}

impl StdError for Error {}

pub type Result<T> = result::Result<T, Error>;

struct Text {
    string: String,
    context: &'static str,
}

impl Text {
    fn new(text: &str, description: &'static str) -> Self {
        Self {
            string: text.trim().replace('\r', "\n"),
            context: description,
        }
    }

    fn error(&self, kind: ErrorKind) -> Error {
        Error {
            context: self.context,
            kind,
        }
    }

    fn not_empty(self) -> Result<Self> {
        if self.string.is_empty() {
            Err(self.error(ErrorKind::Empty))
        } else {
            Ok(self)
        }
    }

    fn no_newlines(self) -> Result<Self> {
        if self.string.contains('\n') {
            Err(self.error(ErrorKind::ContainsNewlines))
        } else {
            Ok(self)
        }
    }
}

impl From<Text> for String {
    fn from(value: Text) -> Self {
        value.string
    }
}

trait AsText {
    fn as_text(&self, description: &'static str) -> Text;
}

impl AsText for str {
    fn as_text(&self, description: &'static str) -> Text {
        Text::new(self, description)
    }
}

macro_rules! text {
    ($name:ident, $description:literal $(, $rules:ident )* ) => {
        #[derive(Clone, Debug)]
        #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
        #[cfg_attr(feature = "serde", serde(try_from = "String"))]
        pub struct $name(String);

        impl $name {
            pub fn new(data: &str) -> Result<Self> {
                Ok(Self(data.as_text($description) $( .$rules()? )*.into()))
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(s: &str) -> result::Result<Self, Self::Err> {
                $name::new(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(value: String) -> result::Result<Self, Self::Error> {
                $name::new(&value)
            }
        }
    };
}

text!(Comment, "comment", not_empty);
text!(Description, "description");
text!(PostTitle, "post title", no_newlines);
text!(TagName, "tag name", not_empty, no_newlines);
