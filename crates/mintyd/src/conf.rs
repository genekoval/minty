use axum_unix::Endpoint;
use log::LevelFilter;
use minty_core::conf::{BucketConfig, DatabaseConfig, SearchConfig};
use serde::{Deserialize, Serialize};
use serde_yaml as yaml;
use std::{fs, path::Path};
use timber::Sink;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub database: DatabaseConfig,

    pub objects: BucketConfig,

    pub search: SearchConfig,

    pub http: Http,

    #[serde(default)]
    pub log: Log,

    pub user: Option<String>,
}

impl Config {
    pub fn read(path: &Path) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|err| {
            format!("failed to read config file '{}': {err}", path.display())
        })?;

        yaml::from_str(&data).map_err(|err| {
            format!(
                "failed to deserialize config file '{}': {err}",
                path.display()
            )
        })
    }

    pub fn set_logger(&self) -> Result<(), String> {
        self.log.set_logger()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Http {
    pub listen: Endpoint,
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

    fn set_logger(&self) -> Result<(), String> {
        timber::new()
            .max_level(self.level)
            .sink(self.sink.clone())
            .init()
            .map_err(|err| format!("failed to initialize logger: {err}"))
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
