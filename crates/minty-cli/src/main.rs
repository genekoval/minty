use minty_cli::*;

use clap::Parser;
use minty::{Pagination, TagQuery, Uuid};
use std::process::ExitCode;

type Result = minty_cli::Result<()>;

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            err.report()
        }
    }
}

fn real_main() -> Result {
    let args = Cli::parse();
    let config = args.config()?;

    config.set_logger()?;

    let Some(server) = config.servers.get(&args.server) else {
        return Err(Error::Config(format!(
            "server alias '{}' not defined",
            args.server
        )));
    };

    let client = Client::new(
        &args.server,
        server,
        Output {
            human_readable: args.human_readable,
            json: args.json,
        },
    );

    async_main(args, client)
}

fn async_main(args: Cli, client: Client) -> Result {
    let body = async move { run_command(args, client).await };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to build runtime: {err}"))?
        .block_on(body)
}

async fn run_command(args: Cli, client: Client) -> Result {
    match args.command {
        Command::About => client.about().await,
        Command::Find {
            command,
            from,
            size,
        } => find(command, Pagination { from, size }, client).await,
        Command::New { command } => new(command, client).await,
        Command::Tag { id, command } => tag(id, command, client).await,
    }
}

async fn find(command: Find, pagination: Pagination, client: Client) -> Result {
    match command {
        Find::Tag { name } => {
            client
                .get_tags(TagQuery {
                    pagination,
                    name,
                    exclude: Default::default(),
                })
                .await
        }
    }
}

async fn new(command: New, client: Client) -> Result {
    match command {
        New::Tag { name } => client.add_tag(&name).await,
    }
}

async fn tag(id: Uuid, command: Option<Tag>, client: Client) -> Result {
    let Some(command) = command else {
        client.get_tag(id).await?;
        return Ok(());
    };

    match command {
        Tag::Aka { alias } => client.add_tag_alias(id, &alias).await,
        Tag::Desc { description } => {
            client.set_tag_description(id, description).await
        }
        Tag::Ln { url } => client.add_tag_source(id, &url).await,
        Tag::Rename { name } => client.set_tag_name(id, &name).await,
        Tag::Rm { force, command } => match command {
            Some(command) => tag_rm(id, command, client).await,
            None => client.delete_tag(id, force).await,
        },
    }
}

async fn tag_rm(id: Uuid, command: TagRm, client: Client) -> Result {
    match command {
        TagRm::Alias { alias } => client.delete_tag_alias(id, alias).await,
        TagRm::Link { sources } => {
            if sources.is_empty() {
                client.delete_tag_source(id).await
            } else {
                client.delete_tag_sources(id, &sources).await
            }
        }
    }
}
