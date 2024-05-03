use crate::{Error, Result};

use log::LevelFilter;
use minty::Url;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};
use timber::Sink;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub log: Log,

    pub servers: HashMap<String, Server>,
}

impl Config {
    pub fn read(path: Option<PathBuf>) -> Result<Self> {
        let Some(path) = path.or_else(find_config) else {
            return Err(Error::Config("could not find a config file".into()));
        };

        let data = fs::read_to_string(&path).map_err(|err| {
            Error::Config(format!(
                "failed to read config file '{}': {err}",
                path.display()
            ))
        })?;

        toml::from_str(&data).map_err(|err| {
            Error::Config(format!(
                "config file '{}' contains errors: {err}",
                path.display()
            ))
        })
    }

    pub fn set_logger(&self) -> Result<()> {
        self.log.set_logger()
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub url: Url,
}

fn find_config() -> Option<PathBuf> {
    search_xdg_config_home().or_else(search_home)
}

fn search_home() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;

    let home = Path::new(&home);
    let config = home.join(".config");

    let path = config.join("minty/minty.toml");
    if path.is_file() {
        return Some(path);
    }

    let path = config.join("minty.toml");
    if path.is_file() {
        return Some(path);
    }

    let path = home.join(".minty.toml");
    if path.is_file() {
        return Some(path);
    }

    None
}

fn search_xdg_config_home() -> Option<PathBuf> {
    let config = env::var_os("XDG_CONFIG_HOME")?;

    let path = Path::new(&config).join("minty/minty.toml");

    if path.is_file() {
        return Some(path);
    }

    None
}
