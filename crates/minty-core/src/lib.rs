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
pub use model::{About, Version};
pub use repo::Repo;
pub use task::Task;

pub struct Env {
    #[allow(dead_code)]
    preview: preview::Env,
}

impl Env {
    fn initialize() -> Self {
        Self {
            preview: preview::Env::initialize(),
        }
    }
}

pub fn initialize() -> Env {
    Env::initialize()
}
