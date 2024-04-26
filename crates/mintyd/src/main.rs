use mintyd::*;

use clap::Parser;
use log::error;
use minty_core::{Repo, Task};
use std::{
    io::{stdout, IsTerminal},
    process::ExitCode,
    sync::Arc,
};
use timber::{
    syslog::{self, Facility, LogOption},
    Sink::Syslog,
};
use tokio::task::JoinHandle;

const SYSLOG_IDENTIFIER: &str = "minty";

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

fn run_async(args: &Cli, config: &Config, parent: &mut dmon::Parent) -> Result {
    let body = async {
        let repo = Arc::new(Repo::new(&config.repo).await?);

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
        Command::Prune => {
            let result = repo.prune().await?;

            if result.objects_removed == 0 {
                println!("No objects to prune");
            } else {
                println!(
                    "Removed {} {} freeing {}",
                    result.objects_removed,
                    match result.objects_removed {
                        1 => "object",
                        _ => "objects",
                    },
                    bytesize::to_string(result.space_freed, true)
                );
            }
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
