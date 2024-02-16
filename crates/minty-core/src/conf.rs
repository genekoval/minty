pub use pgtools::{
    ConnectionParameters as DbConnection, PgDump, PgRestore, Psql,
};

use crate::Version;

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

#[derive(Clone, Copy, Debug)]
pub struct RepoConfig<'a> {
    pub version: Version,
    pub database: &'a DatabaseConfig,
}
