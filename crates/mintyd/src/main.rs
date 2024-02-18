use mintyd::{conf::Config, Result};

use clap::{Parser, Subcommand};
use log::error;
use minty_core::{conf::RepoConfig, Repo, Version};
use shadow_rs::shadow;
use std::{path::PathBuf, process::ExitCode, result, sync::Arc};
use timber::{syslog::LogOption, Sink::Syslog};

shadow!(build);

const DEFAULT_CONFIG: &str = match option_env!("MINTYD_DEFAULT_CONFIG") {
    Some(config) => config,
    None => "/etc/minty/minty.yml",
};

const SYSLOG_IDENTIFIER: &str = "minty";

#[derive(Parser)]
#[command(
    version,
    long_version = build::CLAP_LONG_VERSION,
    arg_required_else_help = true
)]
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
    /// Server config file in YAML format
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Save an archive of the database to the specified file
    Dump {
        /// Location of the archive file
        filename: PathBuf,
    },

    /// Initialize the database
    Init {
        /// Delete existing data if necessary
        overwrite: bool,
    },

    /// Update schemas to match the current program version
    Migrate,

    /// Restore database data from a backup
    Restore {
        /// Location of the archive file
        filename: PathBuf,
    },
}

fn main() -> ExitCode {
    let args = Cli::parse();

    let mut config = match Config::read(args.config.as_path()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };

    if let Syslog(syslog) = &mut config.log.sink {
        syslog.identifier = SYSLOG_IDENTIFIER.into();
        syslog.logopt = LogOption::Pid;
    }

    let run = || {
        config.set_logger()?;
        run_async(&args, &config)
    };

    if let Err(err) = run() {
        error!("{err}");

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn version() -> Version {
    Version {
        number: build::PKG_VERSION,
        branch: build::BRANCH,
        build_time: build::BUILD_TIME,
        build_os: build::BUILD_OS,
        build_type: build::BUILD_RUST_CHANNEL,
        commit_hash: build::COMMIT_HASH,
        commit_date: build::COMMIT_DATE,
        rust_version: build::RUST_VERSION,
        rust_channel: build::RUST_CHANNEL,
    }
}

async fn repo(config: &Config) -> result::Result<Arc<Repo>, String> {
    let config = RepoConfig {
        version: version(),
        database: &config.database,
    };

    Ok(Arc::new(Repo::new(config).await?))
}

fn run_async(args: &Cli, config: &Config) -> Result {
    let body = async {
        let repo = repo(config).await?;

        let result = run_command(args, &repo).await;

        repo.shutdown().await;
        result
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to build the runtime: {err}"))?
        .block_on(body)
}

async fn run_command(args: &Cli, repo: &Arc<Repo>) -> Result {
    match &args.command {
        Command::Dump { filename } => repo.dump(filename).await?,
        Command::Init { overwrite } => {
            if *overwrite {
                repo.reset().await?;
            } else {
                repo.init().await?;
            }
        }
        Command::Migrate => repo.migrate().await?,
        Command::Restore { filename } => repo.restore(filename).await?,
    };

    Ok(())
}
