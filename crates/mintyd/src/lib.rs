pub mod conf;
pub mod server;

mod progress;

pub use progress::ProgressBarTask;

use std::{error::Error, result};

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;
pub type Result = result::Result<(), BoxError>;
