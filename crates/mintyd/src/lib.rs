mod cli;
mod conf;

pub mod server;

mod progress;

pub use cli::*;
pub use conf::Config;
pub use progress::ProgressBarTask;

use shadow_rs::shadow;
use std::{error::Error, result};

shadow!(build);

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;
pub type Result = result::Result<(), BoxError>;
