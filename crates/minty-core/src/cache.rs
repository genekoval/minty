mod conf;
mod sessions;

pub use conf::Config;

use sessions::Sessions;

use crate::SessionId;

use ::moka::future as moka;
use minty::Uuid;

#[derive(Debug)]
pub struct Cache {
    sessions: moka::Cache<SessionId, Uuid>,
}

impl Cache {
    pub fn new(config: &Config) -> Self {
        Self {
            sessions: moka::Cache::new(config.sessions()),
        }
    }

    pub fn sessions(&self) -> Sessions {
        Sessions::new(self)
    }
}
