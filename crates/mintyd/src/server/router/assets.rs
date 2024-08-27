#[cfg(feature = "servedir")]
mod servedir;

#[cfg(not(feature = "servedir"))]
mod embed;

macro_rules! assets {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets")
    };
}

use assets;

use super::Router;

#[cfg(feature = "servedir")]
pub use servedir::routes;

#[cfg(not(feature = "servedir"))]
pub use embed::routes;
