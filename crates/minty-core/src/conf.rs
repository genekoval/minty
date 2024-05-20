pub use elasticsearch::params::Refresh;
pub use pgtools::{
    ConnectionParameters as DbConnection, PgDump, PgRestore, Psql,
};

use minty::Url;
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, path::PathBuf};

const DEFAULT_CACHE_SIZE: NonZeroUsize =
    unsafe { NonZeroUsize::new_unchecked(10_000) };

const DEFAULT_SQL_DIRECTORY: &str =
    match option_env!("MINTY_DEFAULT_SQL_DIRECTORY") {
        Some(dir) => dir,
        None => "db",
    };

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default = "CacheConfig::default_cache_size")]
    pub sessions: NonZeroUsize,
}

impl CacheConfig {
    fn default_cache_size() -> NonZeroUsize {
        DEFAULT_CACHE_SIZE
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            sessions: DEFAULT_CACHE_SIZE,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub connection: DbConnection,

    pub max_connections: Option<u32>,

    #[serde(default)]
    pub psql: Psql,

    #[serde(default)]
    pub pg_dump: PgDump,

    #[serde(default)]
    pub pg_restore: PgRestore,

    #[serde(default = "DatabaseConfig::default_sql_directory")]
    pub sql_directory: PathBuf,
}

impl DatabaseConfig {
    fn default_sql_directory() -> PathBuf {
        DEFAULT_SQL_DIRECTORY.into()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BucketConfig {
    pub url: Url,
    pub bucket: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchConfig {
    pub node: Url,
    pub auth: SearchAuth,
    pub namespace: String,
    #[serde(default = "SearchConfig::default_refresh")]
    pub refresh: Refresh,
}

impl SearchConfig {
    fn default_refresh() -> Refresh {
        Refresh::False
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchAuth {
    pub id: String,
    pub api_key: String,
}

impl From<SearchAuth> for elasticsearch::auth::Credentials {
    fn from(value: SearchAuth) -> Self {
        Self::ApiKey(value.id, value.api_key)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    #[serde(default)]
    pub cache: CacheConfig,
    pub database: DatabaseConfig,
    pub objects: BucketConfig,
    pub search: SearchConfig,
}
