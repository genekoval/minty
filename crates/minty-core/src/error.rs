#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    InvalidInput(String),

    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("Elasticsearch client error: {0}")]
    Elasticsearch(#[from] elasticsearch::Error),

    #[error("fstore error: {0}")]
    Fstore(#[from] fstore::Error),

    #[error("{0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
