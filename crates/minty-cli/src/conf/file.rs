use crate::{Error::Config as Error, Result};

use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::{self, Display},
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct File<T> {
    description: &'static str,
    path: PathBuf,
    data: T,
}

impl<T> File<T>
where
    T: Default + DeserializeOwned + Serialize,
{
    pub fn new(description: &'static str, path: PathBuf) -> Self {
        Self {
            description,
            path,
            data: Default::default(),
        }
    }

    pub fn read(description: &'static str, path: PathBuf) -> Result<Self> {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return Ok(Self::new(description, path)),
                _ => {
                    return Err(Error(format!(
                        "failed to read {description} file '{}': {err}",
                        path.display()
                    )))
                }
            },
        };

        let data = toml::from_str(&text).map_err(|err| {
            Error(format!(
                "{description} file '{}' contains errors: {err}",
                path.display()
            ))
        })?;

        Ok(Self {
            description,
            path,
            data,
        })
    }

    pub fn relative(&self, path: &Path) -> PathBuf {
        if path.is_relative() {
            self.path.parent().unwrap().join(path)
        } else {
            path.to_path_buf()
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

impl<T> Display for File<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = self.description;
        let path = self.path.display();

        write!(f, "{description} file '{path}'")
    }
}
