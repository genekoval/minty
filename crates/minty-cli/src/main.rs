use minty_cli::*;

use clap::Parser;
use minty::{Pagination, PostParts, PostQuery, TagQuery, Uuid, Visibility};
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
        Command::Comment { id, command } => comment(id, command, client).await,
        Command::Comments { post } => client.get_comments(post).await,
        Command::Find {
            command,
            from,
            size,
        } => find(command, Pagination { from, size }, client).await,
        Command::New { command } => new(command, client).await,
        Command::Obj { id, command } => object(id, command, client).await,
        Command::Post { id, command } => post(id, command, client).await,
        Command::Reply { comment, content } => {
            client.reply(comment, content).await
        }
        Command::Tag { id, command } => tag(id, command, client).await,
    }
}

async fn comment(id: Uuid, command: Option<Comment>, client: Client) -> Result {
    let Some(command) = command else {
        client.get_comment(id).await?;
        return Ok(());
    };

    match command {
        Comment::Edit { content } => {
            client.set_comment_content(id, content).await
        }
        Comment::Rm { force, recursive } => {
            client.delete_comment(id, force, recursive).await
        }
    }
}

async fn find(command: Find, pagination: Pagination, client: Client) -> Result {
    match command {
        Find::Post {
            drafts,
            sort_by,
            tag,
            text,
        } => {
            client
                .get_posts(PostQuery {
                    pagination,
                    text: text.unwrap_or_default(),
                    tags: tag,
                    visibility: if drafts {
                        Visibility::Draft
                    } else {
                        Visibility::Public
                    },
                    sort: sort_by,
                })
                .await
        }
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
        New::Comment { post, content } => {
            client.add_comment(post, content).await
        }
        New::Post {
            title,
            description,
            draft,
            tag,
            post,
            objects,
        } => {
            let objects = client.add_objects(objects).await?;
            let objects = if objects.is_empty() {
                None
            } else {
                Some(objects)
            };

            client
                .create_post(PostParts {
                    title,
                    description,
                    visibility: if draft {
                        Some(Visibility::Draft)
                    } else {
                        None
                    },
                    objects,
                    posts: post,
                    tags: tag,
                })
                .await
        }
        New::Tag { name } => client.add_tag(name).await,
    }
}

async fn object(id: Uuid, command: Option<Object>, client: Client) -> Result {
    let Some(command) = command else {
        client.get_object(id).await?;
        return Ok(());
    };

    match command {
        Object::Get {
            no_clobber,
            destination,
        } => client.get_object_data(id, no_clobber, destination).await,
    }
}

async fn post(id: Uuid, command: Option<Post>, client: Client) -> Result {
    let Some(command) = command else {
        client.get_post(id).await?;
        return Ok(());
    };

    match command {
        Post::Desc { text } => client.set_post_description(id, text).await,
        Post::Obj {
            destination,
            objects,
        } => client.add_post_objects(id, objects, destination).await,
        Post::Ln { posts } => client.add_related_posts(id, posts).await,
        Post::Publish => client.publish_post(id).await,
        Post::Rm { force, command } => match command {
            Some(command) => post_rm(id, command, client).await,
            None => client.delete_post(id, force).await,
        },
        Post::Tag { tags } => client.add_post_tags(id, tags).await,
        Post::Title { text } => client.set_post_title(id, text).await,
    }
}

async fn post_rm(id: Uuid, command: PostRm, client: Client) -> Result {
    match command {
        PostRm::Obj { objects } => {
            client.delete_post_objects(id, objects).await
        }
        PostRm::Related { posts } => {
            client.delete_related_posts(id, posts).await
        }
        PostRm::Tag { tags } => client.delete_post_tags(id, tags).await,
    }
}

async fn tag(id: Uuid, command: Option<Tag>, client: Client) -> Result {
    let Some(command) = command else {
        client.get_tag(id).await?;
        return Ok(());
    };

    match command {
        Tag::Aka { alias } => client.add_tag_alias(id, alias).await,
        Tag::Desc { description } => {
            client.set_tag_description(id, description).await
        }
        Tag::Ln { url } => client.add_tag_source(id, &url).await,
        Tag::Rename { name } => client.set_tag_name(id, name).await,
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
