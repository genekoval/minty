use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

const DEFAULT_CACHE_SIZE: usize = 10_000;

fn default_cache_size() -> NonZeroUsize {
    NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap()
}

macro_rules! config {
    ($($cache:ident),*) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct Config {
            #[serde(default = "default_cache_size")]
            default: NonZeroUsize,
            $($cache: Option<NonZeroUsize>,)*
        }

        impl Config {
            $(pub fn $cache(&self) -> NonZeroUsize {
                self.$cache.unwrap_or(self.default)
            })*
        }

        impl Default for Config {
            fn default() -> Self {
                Self {
                    default: default_cache_size(),
                    $($cache: None,)*
                }
            }
        }
    };
}

config!(objects, posts, sessions, tags, users);
