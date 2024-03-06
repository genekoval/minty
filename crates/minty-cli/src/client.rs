mod ask;

use crate::{
    conf::Server,
    output::{About, Output, Print},
};

use minty::{http, Repo, Source, TagQuery, Url, Uuid};
use std::{
    fmt::{self, Display},
    io::{stdin, IsTerminal, Read},
};

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

    pub async fn delete_tag(&self, id: Uuid, force: bool) -> Result {
        if stdin().is_terminal() && !force {
            let name = self.repo.get_tag(id).await?.name;
            let prompt = format!("Delete the tag '{name}'?");
            let confirmed = ask::confirm(&prompt)?;

            if !confirmed {
                return Ok(());
            }
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

    pub async fn get_tag(&self, id: Uuid) -> Result {
        self.print(self.repo.get_tag(id).await?)
    }

    pub async fn get_tags(&self, query: TagQuery) -> Result {
        self.print(self.repo.get_tags(&query).await?)
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
