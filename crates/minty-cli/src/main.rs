use minty_cli::*;

use clap::Parser;
use minty::{
    text, Pagination, PostParts, PostQuery, ProfileQuery, Uuid, Visibility,
};
use std::process::ExitCode;

type Result = minty_cli::Result<()>;

struct Client {
    client: minty_cli::Client,
    config: ConfigFile,
}

impl Client {
    fn new(args: &Cli) -> minty_cli::Result<Self> {
        let config = args.config()?;
        config.set_logger()?;

        let Some(server) = config.server(&args.server) else {
            return Err(Error::Config(format!(
                "server alias '{}' not defined",
                args.server
            )));
        };

        let session = if let Some(email) = config.user(&args.user) {
            Some(config.credentials(server, email.as_ref())?)
        } else {
            None
        }
        .flatten();

        let client = minty_cli::Client::new(
            &args.server,
            server,
            session,
            Output {
                human_readable: args.human_readable,
                json: args.json,
            },
        );

        Ok(Self { client, config })
    }

    fn run(&self, args: Cli) -> Result {
        let body = async move { self.run_async(args).await };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| format!("failed to build runtime: {err}"))?
            .block_on(body)
    }

    async fn run_async(&self, args: Cli) -> Result {
        match args.command {
            Command::About => self.client.about().await,
            Command::Comment { id, command } => self.comment(id, command).await,
            Command::Comments { post } => self.client.get_comments(post).await,
            Command::Email { email } => self.client.set_user_email(email).await,
            Command::Export => self.client.export().await,
            Command::Find {
                command,
                from,
                size,
            } => self.find(command, Pagination { from, size }).await,
            Command::Login => self.login(args.user).await,
            Command::Logout => self.logout(args.user).await,
            Command::Me { command } => self.me(command).await,
            Command::New { command } => self.cmd_new(command).await,
            Command::Obj { id, command } => self.object(id, command).await,
            Command::Password => self.client.set_user_password().await,
            Command::Post { id, command } => self.post(id, command).await,
            Command::Reply { comment, content } => {
                self.client.reply(comment, content).await
            }
            Command::Signup { username } => {
                self.sign_up(username, args.user).await
            }
            Command::Tag { id, command } => self.tag(id, command).await,
            Command::User { id } => self.client.get_user(id).await,
        }
    }

    async fn comment(&self, id: Uuid, command: Option<Comment>) -> Result {
        let Some(command) = command else {
            self.client.get_comment(id).await?;
            return Ok(());
        };

        match command {
            Comment::Edit { content } => {
                self.client.set_comment_content(id, content).await
            }
            Comment::Rm { force, recursive } => {
                self.client.delete_comment(id, force, recursive).await
            }
        }
    }

    async fn find(&self, command: Find, pagination: Pagination) -> Result {
        match command {
            Find::Post {
                drafts,
                poster,
                sort_by,
                tag,
                text,
            } => {
                self.client
                    .get_posts(PostQuery {
                        pagination,
                        poster,
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
                self.client
                    .get_tags(ProfileQuery {
                        pagination,
                        name,
                        exclude: Default::default(),
                    })
                    .await
            }
            Find::User { name } => {
                self.client
                    .get_users(ProfileQuery {
                        pagination,
                        name,
                        exclude: Default::default(),
                    })
                    .await
            }
        }
    }

    async fn login(&self, user: String) -> Result {
        let Some(email) = self.config.user(&user) else {
            return Err(Error::Config(format!(
                "user alias '{}' not defined",
                user
            )));
        };

        let email = email.to_string();
        let session = self.client.authenticate(email.clone()).await?;
        let server = self.client.url().clone();

        self.config.set_credentials(server, email, session)?;

        Ok(())
    }

    async fn logout(&self, user: String) -> Result {
        self.client.sign_out().await?;

        let email = self.config.user(&user).unwrap();
        let server = self.client.url();

        self.config.remove_credentials(server, email.as_ref())
    }

    async fn me(&self, command: Option<Me>) -> Result {
        let Some(command) = command else {
            self.client.get_authenticated_user().await?;
            return Ok(());
        };

        match command {
            Me::Aka { alias } => self.client.add_user_alias(alias).await,
            Me::Desc { description } => {
                self.client.set_user_description(description).await
            }
            Me::Ln { url } => self.client.add_user_source(&url).await,
            Me::Rename { name } => self.client.set_user_name(name).await,
            Me::Rm { force, command } => match command {
                Some(command) => self.me_rm(command).await,
                None => self.client.delete_user(force).await,
            },
        }
    }

    async fn me_rm(&self, command: MeRm) -> Result {
        match command {
            MeRm::Alias { alias } => self.client.delete_user_alias(alias).await,
            MeRm::Link { sources } => {
                if sources.is_empty() {
                    self.client.delete_user_source().await
                } else {
                    self.client.delete_user_sources(&sources).await
                }
            }
        }
    }

    async fn cmd_new(&self, command: New) -> Result {
        match command {
            New::Comment { post, content } => {
                self.client.add_comment(post, content).await
            }
            New::Post {
                title,
                description,
                draft,
                tag,
                post,
                objects,
            } => {
                let objects = self.client.add_objects(objects).await?;
                let objects = if objects.is_empty() {
                    None
                } else {
                    Some(objects)
                };

                self.client
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
            New::Tag { name } => self.client.add_tag(name).await,
        }
    }

    async fn object(&self, id: Uuid, command: Option<Object>) -> Result {
        let Some(command) = command else {
            self.client.get_object(id).await?;
            return Ok(());
        };

        match command {
            Object::Get {
                no_clobber,
                destination,
            } => {
                self.client
                    .get_object_data(id, no_clobber, destination)
                    .await
            }
        }
    }

    async fn post(&self, id: Uuid, command: Option<Post>) -> Result {
        let Some(command) = command else {
            self.client.get_post(id).await?;
            return Ok(());
        };

        match command {
            Post::Desc { text } => {
                self.client.set_post_description(id, text).await
            }
            Post::Obj {
                destination,
                objects,
            } => self.client.add_post_objects(id, objects, destination).await,
            Post::Ln { posts } => {
                self.client.add_related_posts(id, posts).await
            }
            Post::Publish => self.client.publish_post(id).await,
            Post::Rm { force, command } => match command {
                Some(command) => self.post_rm(id, command).await,
                None => self.client.delete_post(id, force).await,
            },
            Post::Tag { tags } => self.client.add_post_tags(id, tags).await,
            Post::Title { text } => self.client.set_post_title(id, text).await,
        }
    }

    async fn post_rm(&self, id: Uuid, command: PostRm) -> Result {
        match command {
            PostRm::Obj { objects } => {
                self.client.delete_post_objects(id, objects).await
            }
            PostRm::Related { posts } => {
                self.client.delete_related_posts(id, posts).await
            }
            PostRm::Tag { tags } => {
                self.client.delete_post_tags(id, tags).await
            }
        }
    }

    async fn sign_up(&self, name: text::Name, user: String) -> Result {
        let Some(email) = self.config.user(&user) else {
            return Err(Error::Config(format!(
                "user alias '{}' not defined",
                user
            )));
        };

        let session = self.client.sign_up(name, email.clone()).await?;
        let server = self.client.url().clone();
        let email = email.to_string();

        self.config.set_credentials(server, email, session)?;

        Ok(())
    }

    async fn tag(&self, id: Uuid, command: Option<Tag>) -> Result {
        let Some(command) = command else {
            self.client.get_tag(id).await?;
            return Ok(());
        };

        match command {
            Tag::Aka { alias } => self.client.add_tag_alias(id, alias).await,
            Tag::Desc { description } => {
                self.client.set_tag_description(id, description).await
            }
            Tag::Ln { url } => self.client.add_tag_source(id, &url).await,
            Tag::Rename { name } => self.client.set_tag_name(id, name).await,
            Tag::Rm { force, command } => match command {
                Some(command) => self.tag_rm(id, command).await,
                None => self.client.delete_tag(id, force).await,
            },
        }
    }

    async fn tag_rm(&self, id: Uuid, command: TagRm) -> Result {
        match command {
            TagRm::Alias { alias } => {
                self.client.delete_tag_alias(id, alias).await
            }
            TagRm::Link { sources } => {
                if sources.is_empty() {
                    self.client.delete_tag_source(id).await
                } else {
                    self.client.delete_tag_sources(id, &sources).await
                }
            }
        }
    }
}

fn real_main() -> Result {
    let args = Cli::parse();
    Client::new(&args)?.run(args)
}

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            err.report()
        }
    }
}
