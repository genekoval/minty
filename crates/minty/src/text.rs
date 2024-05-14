#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use regex::Regex;
use std::{
    error::Error as StdError,
    fmt::{self, Display},
    result,
    str::FromStr,
    sync::OnceLock,
};

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    ContainsNewlines,
    Empty,
    Invalid,
    TooShort(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct Error {
    pub context: &'static str,
    pub kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        let ctx = self.context;

        match self.kind {
            ContainsNewlines => {
                write!(f, "{ctx} must not contain newlines")
            }
            Empty => write!(f, "{ctx} must not be empty"),
            Invalid => write!(f, "invalid {ctx}"),
            TooShort(min) => {
                write!(f, "{ctx} must be at least {min} characters")
            }
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

    fn password_length(self) -> Result<Self> {
        const MIN: usize = 8;

        let len = self.string.len();

        if len < MIN {
            Err(self.error(ErrorKind::TooShort(MIN)))
        } else {
            Ok(self)
        }
    }

    fn valid_email(self) -> Result<Self> {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        let re = REGEX.get_or_init(|| Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap());

        if re.is_match(&self.string) {
            Ok(self)
        } else {
            Err(self.error(ErrorKind::Invalid))
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
                Self::new(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(value: String) -> result::Result<Self, Self::Error> {
                Self::new(&value)
            }
        }
    };
}

text!(Comment, "comment", not_empty);
text!(Description, "description");
text!(Email, "email address", valid_email);
text!(Name, "name", not_empty, no_newlines);
text!(Password, "password", no_newlines, password_length);
text!(PostTitle, "post title", no_newlines);
