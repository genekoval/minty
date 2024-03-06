use crate::{Config, Result};

use clap::{Parser, Subcommand};
use minty::{Url, Uuid};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help = true)]
/// A Minty client for the command line
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

    /// Search for things
    Find {
        #[arg(
            short,
            long,
            value_name = "OFFSET",
            default_value = "0",
            global = true
        )]
        /// Starting entry offset
        from: u32,

        #[arg(
            short,
            long,
            value_name = "LIMIT",
            env = "MINTY_LIMIT",
            default_value = "50",
            global = true
        )]
        /// Maximum number of hits to return
        size: u32,

        #[command(subcommand)]
        command: Find,
    },

    /// Create new tags, posts, etc.
    New {
        #[command(subcommand)]
        command: New,
    },

    /// Read about or modify a tag
    Tag {
        /// Tag ID
        id: Uuid,

        #[command(subcommand)]
        command: Option<Tag>,
    },
}

#[derive(Debug, Subcommand)]
#[command(flatten_help = true)]
pub enum Find {
    /// Search for tags
    Tag {
        /// Name or alias of the tag to search for
        name: String,
    },
}

#[derive(Debug, Subcommand)]
#[command(flatten_help = true)]
pub enum New {
    /// Create a new tag
    Tag {
        /// New tag's name
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum Tag {
    /// Add a tag alias
    Aka {
        /// Tag's new alias
        alias: String,
    },

    /// Set a tag's description
    Desc {
        /// Tag's description
        description: Option<String>,
    },

    /// Add a link to a tag
    Ln {
        /// Tag's new link
        url: Url,
    },

    /// Set a tag's primary name
    Rename {
        /// Tag's new primary name
        name: String,
    },

    /// Delete a tag
    Rm {
        #[arg(short, long)]
        /// Do not prompt for confirmation before removal
        ///
        /// This is the default behavior if STDIN is not a terminal
        force: bool,

        #[command(subcommand)]
        command: Option<TagRm>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TagRm {
    /// Remove a tag's alias
    Alias {
        /// Tag alias to delete
        alias: Option<String>,
    },

    /// Remove a tag's link
    Link {
        /// Tag links to delete
        sources: Vec<String>,
    },
}

impl Cli {
    pub fn config(&self) -> Result<Config> {
        Config::read(self.config.clone())
    }
}
