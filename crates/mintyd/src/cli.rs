use clap::{Args, Parser, Subcommand};
use minty::Uuid;
use std::path::PathBuf;

const DEFAULT_CONFIG: &str = match option_env!("MINTY_DEFAULT_CONFIG") {
    Some(config) => config,
    None => "minty.toml",
};

#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
/// Minty server
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        env = "MINTYD_CONFIG",
        default_value = DEFAULT_CONFIG,
        global = true
    )]
    /// Server config file in TOML format
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Save an archive of the database to the specified file
    Dump {
        /// Location of the archive file
        filename: PathBuf,
    },

    /// Initialize the database
    Init {
        #[arg(short, long)]
        /// Delete existing data if necessary
        overwrite: bool,
    },

    /// Update schemas to match the current program version
    Migrate,

    /// Delete unused data
    Prune,

    /// Restore database data from a backup
    Restore {
        /// Location of the archive file
        filename: PathBuf,
    },

    /// Regenerate assets
    Regen {
        #[command(subcommand)]
        command: Regen,
    },

    /// Rebuild all search engine indices
    Reindex {
        #[arg(short, long, default_value = "100", global = true)]
        /// Max items to upload to search engine in a single request
        batch_size: usize,

        #[arg(short, long, global = true)]
        /// Do not display progress
        quiet: bool,

        #[command(subcommand)]
        command: Option<Reindex>,
    },

    /// Grant administrator privileges to a user
    Admin {
        /// The user's ID
        id: Uuid,
    },

    /// Start the web server
    Serve {
        #[arg(short, long)]
        /// Run the server as a daemon process
        daemon: bool,

        #[arg(short, long, requires = "daemon")]
        /// Path to the pidfile
        pidfile: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum Regen {
    /// Regenerate object previews
    Previews {
        /// ID of object for which to regenerate preview
        object: Option<Uuid>,

        #[command(subcommand)]
        command: Option<RegenPreviews>,
    },
}

#[derive(Subcommand)]
pub enum RegenPreviews {
    All(RegenPreviewsAll),
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct RegenPreviewsAll {
    #[arg(short, long, default_value = "100")]
    pub batch_size: usize,

    #[arg(short, long, default_value = "32")]
    pub max_tasks: usize,
}

#[derive(Subcommand)]
pub enum Reindex {
    /// Reindex all posts
    Posts,

    /// Reindex all tags
    Tags,

    /// Reindex all users
    Users,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
