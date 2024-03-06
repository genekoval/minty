pub use pgtools::{
    ConnectionParameters as DbConnection, PgDump, PgRestore, Psql,
};

use crate::Version;

use minty::Url;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

    pub sql_directory: PathBuf,
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

#[derive(Clone, Copy, Debug)]
pub struct RepoConfig<'a> {
    pub version: Version,
    pub objects: &'a BucketConfig,
    pub database: &'a DatabaseConfig,
    pub search: &'a SearchConfig,
}
