#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
