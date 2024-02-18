pub mod conf;
pub mod server;

use std::{error::Error, result};

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;
pub type Result = result::Result<(), BoxError>;
