mod ask;

use crate::{
    conf::Server,
    output::{About, Output, Print},
};

use minty::{http, model::*, Repo};
use std::{
    fmt::{self, Display},
    io::{stdin, IsTerminal, Read},
    path::PathBuf,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub type Result = crate::Result<()>;

pub struct Client {
    server: String,
    repo: http::Repo,
    output: Output,
}

impl Client {
    pub fn new(alias: &str, server: &Server, output: Output) -> Self {
        Self {
            server: alias.into(),
            repo: http::Repo::new(&server.url),
            output,
        }
    }

    fn print<T: Print>(&self, t: T) -> Result {
        Ok(t.print(self.output)?)
    }

    pub async fn about(&self) -> Result {
        let info = self.repo.about().await?;
        let about = About {
            server: &self.server,
            url: self.repo.url(),
            info,
        };

        self.print(about)
    }

    pub async fn add_comment(
        &self,
        post: Uuid,
        content: Option<String>,
    ) -> Result {
        let content = match content {
            Some(content) => content,
            None => read_from_stdin()?,
        };

        let comment = self.repo.add_comment(post, content.trim()).await?;
        println!("{}", comment.id);

        Ok(())
    }

    pub async fn add_objects(
        &self,
        args: Vec<String>,
    ) -> crate::Result<Vec<Uuid>> {
        let mut objects = Vec::new();

        for arg in args {
            if let Ok(uuid) = Uuid::parse_str(&arg) {
                objects.push(uuid);
            } else if let Ok(url) = Url::parse(&arg) {
                objects.push(self.upload_url(url).await?);
            } else {
                let path = PathBuf::from(&arg);
                objects.push(self.upload_file(path).await?);
            }
        }

        Ok(objects)
    }

    pub async fn add_post_objects(
        &self,
        post_id: Uuid,
        objects: Vec<String>,
        destination: Option<Uuid>,
    ) -> Result {
        let objects = self.add_objects(objects).await?;

        match destination {
            Some(destination) => {
                self.repo
                    .insert_post_objects(post_id, &objects, destination)
                    .await?
            }
            None => self.repo.append_post_objects(post_id, &objects).await?,
        };

        Ok(())
    }

    pub async fn add_post_tags(
        &self,
        post_id: Uuid,
        tags: Vec<Uuid>,
    ) -> Result {
        for tag in tags {
            self.repo.add_post_tag(post_id, tag).await?;
        }

        Ok(())
    }

    pub async fn add_related_posts(
        &self,
        post_id: Uuid,
        posts: Vec<Uuid>,
    ) -> Result {
        for post in posts {
            self.repo.add_related_post(post_id, post).await?;
        }

        Ok(())
    }

    pub async fn add_tag(&self, name: &str) -> Result {
        let id = self.repo.add_tag(name).await?;
        println!("{id}");
        Ok(())
    }

    pub async fn add_tag_alias(&self, id: Uuid, alias: &str) -> Result {
        self.print(self.repo.add_tag_alias(id, alias).await?)
    }

    pub async fn add_tag_source(&self, id: Uuid, url: &Url) -> Result {
        self.repo.add_tag_source(id, url).await?;
        Ok(())
    }

    pub async fn create_post(&self, parts: PostParts) -> Result {
        let id = self.repo.create_post(&parts).await?;
        println!("{id}");
        Ok(())
    }

    pub async fn delete_comment(
        &self,
        id: Uuid,
        force: bool,
        recursive: bool,
    ) -> Result {
        if stdin().is_terminal() && !force {
            let prompt = if recursive {
                format!(
                    "Delete the comment with ID {id} and any child comments?"
                )
            } else {
                format!("Delete the comment with ID {id}?")
            };

            ask::confirm!(&prompt)?;
        }

        self.repo.delete_comment(id, recursive).await?;
        Ok(())
    }

    pub async fn delete_post(&self, id: Uuid, force: bool) -> Result {
        if stdin().is_terminal() && !force {
            let post = self.repo.get_post(id).await?;
            let title = post.title.as_str();
            let ty = match post.visibility {
                Visibility::Draft => "draft",
                _ => "post",
            };

            let prompt = if title.is_empty() {
                format!("Delete the {ty} with ID {id}?")
            } else {
                format!("Delete the {ty} titled '{title}'?")
            };

            ask::confirm!(&prompt)?;
        }

        self.repo.delete_post(id).await?;
        Ok(())
    }

    pub async fn delete_post_objects(
        &self,
        post_id: Uuid,
        objects: Vec<Uuid>,
    ) -> Result {
        self.repo.delete_post_objects(post_id, &objects).await?;
        Ok(())
    }

    pub async fn delete_post_tags(
        &self,
        post_id: Uuid,
        tags: Vec<Uuid>,
    ) -> Result {
        for tag in tags {
            self.repo.delete_post_tag(post_id, tag).await?;
        }

        Ok(())
    }

    pub async fn delete_related_posts(
        &self,
        post_id: Uuid,
        posts: Vec<Uuid>,
    ) -> Result {
        for post in posts {
            self.repo.delete_related_post(post_id, post).await?;
        }

        Ok(())
    }

    pub async fn delete_tag(&self, id: Uuid, force: bool) -> Result {
        if stdin().is_terminal() && !force {
            let name = self.repo.get_tag(id).await?.name;
            let prompt = format!("Delete the tag '{name}'?");

            ask::confirm!(&prompt)?;
        }

        self.repo.delete_tag(id).await?;
        Ok(())
    }

    pub async fn delete_tag_alias(
        &self,
        id: Uuid,
        alias: Option<String>,
    ) -> Result {
        let alias = match alias {
            Some(alias) => alias,
            None => {
                let tag = self.repo.get_tag(id).await?;

                println!("{}", tag.name);

                match ask::multiple_choice(
                    tag.aliases,
                    "Which alias would you like to remove?",
                )? {
                    Some(alias) => alias,
                    None => return Ok(()),
                }
            }
        };

        self.print(self.repo.delete_tag_alias(id, &alias).await?)
    }

    pub async fn delete_tag_source(&self, id: Uuid) -> Result {
        let tag = self.repo.get_tag(id).await?;

        if tag.sources.is_empty() {
            println!("tag '{}' has no links", tag.name);
            return Ok(());
        }

        let sources: Vec<SourceChoice> =
            tag.sources.into_iter().map(Into::into).collect();

        let source = match ask::multiple_choice(
            sources,
            "Which source would you like to remove?",
        )? {
            Some(source) => source,
            None => return Ok(()),
        };

        self.repo.delete_tag_source(id, source.id).await?;

        Ok(())
    }

    pub async fn delete_tag_sources(
        &self,
        id: Uuid,
        sources: &[String],
    ) -> Result {
        self.repo.delete_tag_sources(id, sources).await?;
        Ok(())
    }

    pub async fn get_comment(&self, id: Uuid) -> Result {
        self.print(self.repo.get_comment(id).await?)
    }

    pub async fn get_comments(&self, post_id: Uuid) -> Result {
        self.print(self.repo.get_comments(post_id).await?)
    }

    pub async fn get_post(&self, id: Uuid) -> Result {
        self.print(self.repo.get_post(id).await?)
    }

    pub async fn get_posts(&self, query: PostQuery) -> Result {
        self.print(self.repo.get_posts(&query).await?)
    }

    pub async fn get_tag(&self, id: Uuid) -> Result {
        self.print(self.repo.get_tag(id).await?)
    }

    pub async fn get_tags(&self, query: TagQuery) -> Result {
        self.print(self.repo.get_tags(&query).await?)
    }

    pub async fn publish_post(&self, id: Uuid) -> Result {
        self.repo.publish_post(id).await?;
        Ok(())
    }

    pub async fn reply(&self, parent: Uuid, content: Option<String>) -> Result {
        let content = match content {
            Some(content) => content,
            None => read_from_stdin()?,
        };

        let comment = self.repo.add_reply(parent, content.trim()).await?;
        println!("{}", comment.id);

        Ok(())
    }

    pub async fn set_comment_content(
        &self,
        id: Uuid,
        content: Option<String>,
    ) -> Result {
        let content = match content {
            Some(content) => content,
            None => read_from_stdin()?,
        };

        self.repo.set_comment_content(id, content.trim()).await?;

        Ok(())
    }

    pub async fn set_post_description(
        &self,
        id: Uuid,
        description: Option<String>,
    ) -> Result {
        let description = match description {
            Some(description) => description,
            None => read_from_stdin()?,
        };

        self.repo
            .set_post_description(id, description.trim())
            .await?;

        Ok(())
    }

    pub async fn set_post_title(
        &self,
        id: Uuid,
        title: Option<String>,
    ) -> Result {
        let title = match title {
            Some(title) => title,
            None => read_from_stdin()?,
        };

        self.repo.set_post_title(id, title.trim()).await?;

        Ok(())
    }

    pub async fn set_tag_description(
        &self,
        id: Uuid,
        description: Option<String>,
    ) -> Result {
        let description = match description {
            Some(description) => description,
            None => read_from_stdin()?,
        };

        self.repo
            .set_tag_description(id, description.trim())
            .await?;

        Ok(())
    }

    pub async fn set_tag_name(&self, id: Uuid, name: &str) -> Result {
        self.print(self.repo.set_tag_name(id, name).await?)
    }

    async fn upload_file(&self, path: PathBuf) -> crate::Result<Uuid> {
        let file = File::open(&path).await.map_err(|err| {
            format!("failed to open file '{}': {err}", path.display())
        })?;
        let stream = ReaderStream::new(file);
        let object = self.repo.add_object(stream).await?;

        Ok(object.id)
    }

    async fn upload_url(&self, url: Url) -> crate::Result<Uuid> {
        let stream = reqwest::get(url.as_str())
            .await
            .map_err(|err| format!("request to '{url}' failed: {err}"))?
            .error_for_status()
            .map_err(|err| format!("({}) {url}", err.status().unwrap()))?
            .bytes_stream();

        let object = self.repo.add_object(stream).await?;

        Ok(object.id)
    }
}

#[derive(Default)]
struct SourceChoice {
    id: i64,
    url: String,
}

impl From<Source> for SourceChoice {
    fn from(value: Source) -> Self {
        Self {
            id: value.id,
            url: value.url.to_string(),
        }
    }
}

impl Display for SourceChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.url)
    }
}

fn read_from_stdin() -> crate::Result<String> {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
