pub mod conf;

mod comment;
mod db;
mod error;
mod ico;
mod model;
mod obj;
mod preview;
mod repo;
mod search;
mod task;

pub use error::{Error, Result};
pub use model::About;
pub use repo::Repo;
pub use task::Task;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Env {
    _preview: preview::Env,
}

impl Env {
    fn initialize() -> Self {
        Self {
            _preview: preview::Env::initialize(),
        }
    }
}

pub fn initialize() -> Env {
    Env::initialize()
}
