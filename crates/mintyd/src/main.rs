use mintyd::{conf::Config, server, ProgressBarTask, Result};

use clap::{Args, Parser, Subcommand};
use log::error;
use minty::Uuid;
use minty_core::{conf::RepoConfig, Repo, Task, Version};
use shadow_rs::shadow;
use std::{
    io::{stdout, IsTerminal},
    path::PathBuf,
    process::ExitCode,
    result,
    sync::Arc,
};
use timber::{
    syslog::{self, Facility, LogOption},
    Sink::Syslog,
};
use tokio::task::JoinHandle;

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
enum Regen {
    /// Regenerate object previews
    Previews {
        /// ID of object for which to regenerate preview
        object: Option<Uuid>,

        #[command(subcommand)]
        command: Option<RegenPreviews>,
    },
}

#[derive(Subcommand)]
enum RegenPreviews {
    All(RegenPreviewsAll),
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
struct RegenPreviewsAll {
    #[arg(short, long, default_value = "100")]
    batch_size: usize,

    #[arg(short, long, default_value = "32")]
    max_tasks: usize,
}

#[derive(Subcommand)]
enum Reindex {
    /// Reindex all posts
    Posts,

    /// Reindex all tags
    Tags,
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

    let mut parent = dmon::Parent::default();

    if let Command::Serve { daemon, pidfile } = &args.command {
        if *daemon {
            config.log.sink = Syslog(syslog::Config {
                identifier: SYSLOG_IDENTIFIER.into(),
                logopt: LogOption::Pid,
                facility: Facility::Daemon,
            });

            parent = dmon::options()
                .pidfile(pidfile.as_deref())
                .permissions(config.user.as_deref())
                .daemonize();
        }
    } else if let Syslog(syslog) = &mut config.log.sink {
        syslog.identifier = SYSLOG_IDENTIFIER.into();
        syslog.logopt = LogOption::Pid;
    }

    let mut run = || {
        config.set_logger()?;
        let _env = minty_core::initialize();
        run_async(&args, &config, &mut parent)
    };

    if let Err(err) = run() {
        error!("{err}");

        if parent.is_waiting() {
            if let Err(write_error) = parent.write(&format!("{err}")) {
                error!("Failed to write to parent process: {write_error}");
            }
        }

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
        objects: &config.objects,
        database: &config.database,
        search: &config.search,
    };

    Ok(Arc::new(Repo::new(config).await?))
}

fn run_async(args: &Cli, config: &Config, parent: &mut dmon::Parent) -> Result {
    let body = async {
        let repo = repo(config).await?;

        let result = run_command(args, config, parent, &repo).await;

        repo.shutdown().await;
        result
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to build the runtime: {err}"))?
        .block_on(body)
}

async fn run_command(
    args: &Cli,
    config: &Config,
    parent: &mut dmon::Parent,
    repo: &Arc<Repo>,
) -> Result {
    match &args.command {
        Command::Dump { filename } => repo.dump(filename).await?,
        Command::Init { overwrite } => {
            if *overwrite {
                repo.reset().await?;
            } else {
                repo.init().await?;
            }

            repo.create_indices().await?;
        }
        Command::Migrate => repo.migrate().await?,
        Command::Regen { command } => match command {
            Regen::Previews { object, command } => match object {
                Some(object) => {
                    let preview = repo.regenerate_preview(*object).await?;
                    match preview {
                        Some(id) => println!("{id}"),
                        None => println!("<no preview>"),
                    }
                }
                None => match command.as_ref().unwrap() {
                    RegenPreviews::All(args) => {
                        regenerate_previews(repo, args).await?
                    }
                },
            },
        },
        Command::Reindex {
            batch_size,
            quiet,
            command,
        } => match command {
            Some(index) => match index {
                Reindex::Posts => {
                    reindex_posts(repo, *batch_size, *quiet).await?
                }
                Reindex::Tags => {
                    reindex_tags(repo, *batch_size, *quiet).await?
                }
            },
            None => {
                reindex_posts(repo, *batch_size, *quiet).await?;
                reindex_tags(repo, *batch_size, *quiet).await?;
            }
        },
        Command::Restore { filename } => repo.restore(filename).await?,
        Command::Serve { .. } => {
            server::serve(&config.http, repo.clone(), parent).await?
        }
    };

    Ok(())
}

async fn regenerate_previews(
    repo: &Arc<Repo>,
    args: &RegenPreviewsAll,
) -> Result {
    let (task, handle) = repo
        .regenerate_previews(args.batch_size, args.max_tasks)
        .await?;

    let title = "Regenerating object previews".into();
    let progress = match ProgressBarTask::new(title, task) {
        Ok(progress) => Some(progress),
        Err(err) => {
            eprintln!("{err}");
            None
        }
    };

    let result = handle.await;

    if let Some(progress) = progress {
        if let Err(err) = progress.join().await {
            eprintln!("{err}");
        }
    }

    Ok(result??)
}

async fn reindex(
    index: &str,
    quiet: bool,
    (task, handle): (Task, JoinHandle<minty_core::Result<()>>),
) -> Result {
    let progress = if quiet || !stdout().is_terminal() {
        None
    } else {
        let title = format!(
            "Indexing {} {}{}",
            task.total(),
            index,
            match task.total() {
                1 => "",
                _ => "s",
            }
        );

        match ProgressBarTask::new(title, task.clone()) {
            Ok(progress) => Some(progress),
            Err(err) => {
                eprintln!("{err}");
                None
            }
        }
    };

    let result = handle.await;

    if let Some(progress) = progress {
        if let Err(err) = progress.join().await {
            eprintln!("{err}");
        }
    }

    result??;

    println!(
        "Indexed {} {}{} in {}ms",
        task.total(),
        index,
        match task.total() {
            1 => "",
            _ => "s",
        },
        task.elapsed().num_milliseconds()
    );

    Ok(())
}

async fn reindex_posts(
    repo: &Arc<Repo>,
    batch_size: usize,
    quiet: bool,
) -> Result {
    reindex("post", quiet, repo.reindex_posts(batch_size).await?).await
}

async fn reindex_tags(
    repo: &Arc<Repo>,
    batch_size: usize,
    quiet: bool,
) -> Result {
    reindex("tag", quiet, repo.reindex_tags(batch_size).await?).await
}
