use crate::cli::{Error, Result};

use minty::Url;
use serde::{Deserialize, Serialize};
use serde_yaml as yaml;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
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

        yaml::from_str(&data).map_err(|err| {
            Error::Config(format!(
                "config file '{}' contains errors: {err}",
                path.display()
            ))
        })
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

    let path = config.join("minty/minty.yml");
    if path.is_file() {
        return Some(path);
    }

    let path = config.join("minty.yml");
    if path.is_file() {
        return Some(path);
    }

    let path = home.join(".minty.yml");
    if path.is_file() {
        return Some(path);
    }

    None
}

fn search_xdg_config_home() -> Option<PathBuf> {
    let config = env::var_os("XDG_CONFIG_HOME")?;

    let path = Path::new(&config).join("minty/minty.yml");

    if path.is_file() {
        return Some(path);
    }

    None
}
