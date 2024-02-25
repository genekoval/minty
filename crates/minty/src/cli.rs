mod client;
mod conf;
mod error;
mod output;

pub use client::Client;
pub use conf::Config;
pub use error::*;
pub use output::Output;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        env = "MINTY_CONFIG",
        global = true
    )]
    /// Config file in YAML format
    pub config: Option<PathBuf>,

    #[arg(
        long,
        value_name = "ALIAS",
        env = "MINTY_SERVER",
        global = true,
        default_value = "default"
    )]
    /// The configured server to use
    pub server: String,

    #[arg(short = 'H', long, env = "MINTY_HUMAN_READABLE", global = true)]
    /// Print data in a human-readable format
    pub human_readable: bool,

    #[arg(short, long, env = "MINTY_JSON", global = true)]
    /// Print data in JSON format
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get detailed information about the server
    About,
}

impl Cli {
    pub fn config(&self) -> Result<Config> {
        Config::read(self.config.clone())
    }
}
