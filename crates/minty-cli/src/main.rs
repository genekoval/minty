use minty_cli::{Cli, Client, Command, Error, Output, TagArgs};

use clap::Parser;
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
        Command::Tag(args) => tag(args, client).await,
    }
}

async fn tag(args: TagArgs, client: Client) -> Result {
    client.get_tag(args.id).await
}
