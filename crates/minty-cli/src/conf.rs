mod credentials;
mod file;

use credentials::Credentials;
use file::File;

use crate::{Error, Result};

use log::LevelFilter;
use minty::{text::Email, Url};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};
use timber::Sink;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub credentials: Option<PathBuf>,

    #[serde(default)]
    pub log: Log,

    pub servers: BTreeMap<String, Url>,

    pub users: BTreeMap<String, Email>,
}

#[derive(Debug)]
pub struct ConfigFile(File<Config>);

impl ConfigFile {
    pub fn read(path: Option<PathBuf>) -> Result<Self> {
        let Some(path) = path.or_else(find_config) else {
            return Err(Error::Config("could not find a config file".into()));
        };

        let file = File::read("config", path)?;

        Ok(Self(file))
    }

    pub fn set_logger(&self) -> Result<()> {
        self.0.data().log.set_logger()
    }

    pub fn server(&self, alias: &str) -> Option<&Url> {
        self.0.data().servers.get(alias)
    }

    pub fn user(&self, alias: &str) -> Option<&Email> {
        self.0.data().users.get(alias)
    }

    pub fn credentials(
        &self,
        server: &Url,
        email: &str,
    ) -> Result<Option<String>> {
        Ok(self.read_credentials()?.data().get(server, email))
    }

    pub fn set_credentials(
        &self,
        server: Url,
        email: String,
        secret: String,
    ) -> Result<()> {
        let mut file = self.read_credentials()?;
        file.data_mut().insert(server, email, secret);
        file.write()?;
        file.set_permissions(0o600)
    }

    pub fn remove_credentials(&self, server: &Url, email: &str) -> Result<()> {
        let mut file = self.read_credentials()?;
        let credentials = file.data_mut();

        credentials.remove(server, email);

        if credentials.is_empty() {
            file.remove()
        } else {
            file.write()
        }
    }

    fn read_credentials(&self) -> Result<File<Credentials>> {
        let credentials = self.0.data().credentials.clone();
        self.0.read_relative("credentials", credentials)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Log {
    #[serde(default = "Log::default_level")]
    pub level: LevelFilter,

    #[serde(default)]
    pub sink: Sink,
}

impl Log {
    fn default_level() -> LevelFilter {
        LevelFilter::Info
    }

    fn set_logger(&self) -> Result<()> {
        timber::new()
            .max_level(self.level)
            .sink(self.sink.clone())
            .init()
            .map_err(|err| format!("failed to initialize logger: {err}"))?;

        Ok(())
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            sink: Default::default(),
        }
    }
}

fn find_config() -> Option<PathBuf> {
    search_xdg_config_home().or_else(search_home)
}

fn search_home() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;

    let home = Path::new(&home);
    let config = home.join(".config");

    let path = config.join("minty");
    if path.is_dir() {
        return Some(path);
    }

    None
}

fn search_xdg_config_home() -> Option<PathBuf> {
    let config = env::var_os("XDG_CONFIG_HOME")?;

    let confdir = Path::new(&config).join("minty");
    if confdir.is_dir() {
        return Some(confdir);
    }

    None
}
