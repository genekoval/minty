use minty::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{entity} with ID '{id}' not found")]
    NotFound { entity: &'static str, id: Uuid },

    #[error("{entity} with {identifier} already exists")]
    AlreadyExists {
        entity: &'static str,
        identifier: String,
    },

    #[error("{0}")]
    InvalidInput(String),

    #[error("authentication required{}", match .0 {
        Some(message) => format!(": {message}"),
        None => "".into(),
    })]
    Unauthenticated(Option<&'static str>),

    #[error("permission denied")]
    Unauthorized,

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

pub trait Found {
    type Value;

    fn found(self, entity: &'static str, id: Uuid) -> Result<Self::Value>;
}

impl Found for bool {
    type Value = ();

    fn found(self, entity: &'static str, id: Uuid) -> Result<Self::Value> {
        if self {
            Ok(())
        } else {
            Err(Error::NotFound { entity, id })
        }
    }
}

impl<T> Found for Option<T> {
    type Value = T;

    fn found(self, entity: &'static str, id: Uuid) -> Result<Self::Value> {
        match self {
            Some(value) => Ok(value),
            None => Err(Error::NotFound { entity, id }),
        }
    }
}
