mod file;

use file::File;

use crate::{Error, Result};

use log::LevelFilter;
use minty::Url;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};
use timber::Sink;

const CONFIG_FILE: &str = "minty.toml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub log: Log,

    pub servers: BTreeMap<String, Url>,

    cookies: Option<PathBuf>,
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

    pub fn cookies(&self) -> PathBuf {
        self.0
            .data()
            .cookies
            .clone()
            .unwrap_or_else(|| self.0.relative("cookies.json"))
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
    let path = Path::new(&home).join(".config").join("minty");

    path.is_dir().then(|| path.join(CONFIG_FILE))
}

fn search_xdg_config_home() -> Option<PathBuf> {
    let config = env::var_os("XDG_CONFIG_HOME")?;
    let path = Path::new(&config).join("minty");

    path.is_dir().then(|| path.join(CONFIG_FILE))
}
