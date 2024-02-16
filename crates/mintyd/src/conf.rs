use axum_unix::Endpoint;
use log::LevelFilter;
use minty_core::conf::DatabaseConfig;
use serde::{Deserialize, Serialize};
use serde_yaml as yaml;
use std::{fs::File, path::Path};
use timber::Sink;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub database: DatabaseConfig,

    pub http: Http,

    #[serde(default)]
    pub log: Log,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Http {
    pub listen: Endpoint,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Log {
    #[serde(default = "Log::default_level")]
    pub level: LevelFilter,

    pub sink: Sink,
}

impl Log {
    fn default_level() -> LevelFilter {
        LevelFilter::Info
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

pub fn read(path: &Path) -> Result<Config, String> {
    let file = File::open(path).map_err(|err| {
        format!("failed to open config file '{}': {err}", path.display())
    })?;

    yaml::from_reader(file).map_err(|err| {
        format!(
            "failed to deserialize config file '{}': {err}",
            path.display()
        )
    })?
}
