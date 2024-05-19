use crate::{Error::Config as Error, Result};

use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::{self, Display},
    fs::{self, Permissions},
    io::ErrorKind,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
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

    pub fn write(&self) -> Result<()> {
        let text = toml::to_string_pretty(&self.data).map_err(|err| {
            Error(format!("failed to serialize {self}: {err}"))
        })?;

        fs::write(&self.path, text)
            .map_err(|err| Error(format!("failed to write {self}: {err}")))
    }

    pub fn read_relative<U>(
        &self,
        description: &'static str,
        path: Option<PathBuf>,
    ) -> Result<File<U>>
    where
        U: Default + DeserializeOwned + Serialize,
    {
        let confdir = self.path.parent().unwrap();
        let path = path
            .map(|path| {
                if path.is_relative() {
                    confdir.join(path)
                } else {
                    path
                }
            })
            .unwrap_or_else(|| {
                let mut file = confdir.join(description);
                file.set_extension("toml");
                file
            });

        File::read(description, path)
    }

    pub fn remove(&self) -> Result<()> {
        fs::remove_file(&self.path)
            .map_err(|err| Error(format!("failed to remove {self}: {err}")))
    }

    pub fn set_permissions(&self, mode: u32) -> Result<()> {
        let permissions = Permissions::from_mode(mode);
        fs::set_permissions(&self.path, permissions).map_err(|err| {
            Error(format!("failed to set permissions for {self}: {err}"))
        })
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Display for File<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = self.description;
        let path = self.path.display();

        write!(f, "{description} file '{path}'")
    }
}
