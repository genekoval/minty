use crate::{Config, Result};

use clap::{Args, Parser, Subcommand};
use minty::Uuid;
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

    Tag(TagArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
/// Get information about a tag
pub struct TagArgs {
    /// Tag ID
    pub id: Uuid,
}

impl Cli {
    pub fn config(&self) -> Result<Config> {
        Config::read(self.config.clone())
    }
}
