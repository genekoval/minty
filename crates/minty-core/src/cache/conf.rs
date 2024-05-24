use serde::{Deserialize, Serialize};

const DEFAULT_CACHE_SIZE: u64 = 10_000;

fn default_cache_size() -> u64 {
    DEFAULT_CACHE_SIZE
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_cache_size")]
    default: u64,
    sessions: Option<u64>,
}

impl Config {
    pub fn sessions(&self) -> u64 {
        self.sessions.unwrap_or(self.default)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: DEFAULT_CACHE_SIZE,
            sessions: None,
        }
    }
}
