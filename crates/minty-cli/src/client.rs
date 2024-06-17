mod ask;

use crate::output::{About, Output, Print};

use minty::{http, model::*, text, Repo};
use rpassword::prompt_password;
use serde_json as json;
use std::{
    io::{stdin, IsTerminal, Read},
    path::PathBuf,
    str::FromStr,
};
use tokio::fs::File;
use tokio_util::io::{ReaderStream, StreamReader};

pub type Result = crate::Result<()>;

pub struct Client {
    server: String,
    repo: http::Repo,
    output: Output,
}

impl Client {
    pub fn new(
        alias: &str,
        server: &Url,
        session: Option<String>,
        output: Output,
    ) -> Self {
        Self {
            server: alias.into(),
            repo: http::Repo::new(server, session),
            output,
        }
    }

    pub fn url(&self) -> &Url {
        self.repo.url()
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
        content: Option<text::Comment>,
    ) -> Result {
        let content = match content {
            Some(content) => content,
            None => read_from_stdin()?,
        };

        let comment = self.repo.add_comment(post, content).await?;
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

    pub async fn add_tag(&self, name: text::Name) -> Result {
        let id = self.repo.add_tag(name).await?;
        println!("{id}");
        Ok(())
    }

    pub async fn add_tag_alias(&self, id: Uuid, alias: text::Name) -> Result {
        self.print(self.repo.add_tag_alias(id, alias).await?)
    }

    pub async fn add_tag_source(&self, id: Uuid, url: &Url) -> Result {
        self.repo.add_tag_source(id, url).await?;
        Ok(())
    }

    pub async fn add_user_alias(&self, alias: text::Name) -> Result {
        self.print(self.repo.add_user_alias(alias).await?)
    }

    pub async fn add_user_source(&self, url: &Url) -> Result {
        self.repo.add_user_source(url).await?;
        Ok(())
    }

    pub async fn authenticate(&self, email: String) -> crate::Result<String> {
        let password = prompt_password(format!("Password for '{email}': "))?;
        let login = Login { email, password };
        Ok(self.repo.authenticate(&login).await?)
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
            let name = self.repo.get_tag(id).await?.profile.name;
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
                match ask::delete_alias(tag.profile)? {
                    Some(alias) => alias,
                    None => return Ok(()),
                }
            }
        };

        self.print(self.repo.delete_tag_alias(id, &alias).await?)
    }

    pub async fn delete_tag_source(&self, id: Uuid) -> Result {
        let tag = self.repo.get_tag(id).await?;
        let Some(source) = ask::delete_source("tag", tag.profile)? else {
            return Ok(());
        };

        self.repo.delete_tag_source(id, source).await?;

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

    pub async fn delete_user(&self, force: bool) -> Result {
        if stdin().is_terminal() && !force {
            let name = self.repo.get_authenticated_user().await?.profile.name;
            let prompt =
                format!("Delete the currently logged in user '{name}'?");

            ask::confirm!(&prompt)?;
        }

        self.repo.delete_user().await?;
        Ok(())
    }

    pub async fn delete_user_alias(&self, alias: Option<String>) -> Result {
        let alias = match alias {
            Some(alias) => alias,
            None => {
                let user = self.repo.get_authenticated_user().await?;
                match ask::delete_alias(user.profile)? {
                    Some(alias) => alias,
                    None => return Ok(()),
                }
            }
        };

        self.print(self.repo.delete_user_alias(&alias).await?)
    }

    pub async fn delete_user_source(&self) -> Result {
        let user = self.repo.get_authenticated_user().await?;
        let Some(source) = ask::delete_source("user", user.profile)? else {
            return Ok(());
        };

        self.repo.delete_user_source(source).await?;

        Ok(())
    }

    pub async fn delete_user_sources(&self, sources: &[String]) -> Result {
        self.repo.delete_user_sources(sources).await?;
        Ok(())
    }

    pub async fn export(&self) -> Result {
        let data = self.repo.export().await?;
        let json = json::to_string_pretty(&data).map_err(|err| {
            format!("failed to serialize exported data as JSON: {err}")
        })?;
        println!("{}", json);
        Ok(())
    }

    pub async fn get_authenticated_user(&self) -> Result {
        self.print(self.repo.get_authenticated_user().await?)
    }

    pub async fn get_comment(&self, id: Uuid) -> Result {
        self.print(self.repo.get_comment(id).await?)
    }

    pub async fn get_comments(&self, post_id: Uuid) -> Result {
        self.print(self.repo.get_comments(post_id).await?)
    }

    pub async fn get_object(&self, id: Uuid) -> Result {
        self.print(self.repo.get_object(id).await?)
    }

    pub async fn get_object_data(
        &self,
        id: Uuid,
        no_clobber: bool,
        destination: Option<PathBuf>,
    ) -> Result {
        let (_, stream) = self.repo.get_object_data(id).await?;
        let mut reader = StreamReader::new(stream);

        match destination.as_deref() {
            Some(path) => {
                let mut file = File::options()
                    .create(true)
                    .create_new(no_clobber)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .await
                    .map_err(|err| {
                        format!(
                            "failed to open file '{}': {err}",
                            path.display()
                        )
                    })?;

                tokio::io::copy(&mut reader, &mut file).await.map_err(
                    |err| {
                        format!(
                            "failed to stream data to file '{}': {err}",
                            path.display()
                        )
                    },
                )?;
            }
            None => {
                let mut stdout = tokio::io::stdout();

                tokio::io::copy(&mut reader, &mut stdout).await.map_err(
                    |err| format!("failed to stream data to stdout: {err}"),
                )?;
            }
        }

        Ok(())
    }

    pub async fn get_object_preview_errors(&self) -> Result {
        self.print(self.repo.get_object_preview_errors().await?)
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

    pub async fn get_tags(&self, query: ProfileQuery) -> Result {
        self.print(self.repo.get_tags(&query).await?)
    }

    pub async fn get_user(&self, id: Uuid) -> Result {
        self.print(self.repo.get_user(id).await?)
    }

    pub async fn get_users(&self, query: ProfileQuery) -> Result {
        self.print(self.repo.get_users(&query).await?)
    }

    pub async fn publish_post(&self, id: Uuid) -> Result {
        self.repo.publish_post(id).await?;
        Ok(())
    }

    pub async fn reply(
        &self,
        parent: Uuid,
        content: Option<text::Comment>,
    ) -> Result {
        let content = match content {
            Some(content) => content,
            None => read_from_stdin()?,
        };

        let comment = self.repo.add_reply(parent, content).await?;
        println!("{}", comment.id);

        Ok(())
    }

    pub async fn set_comment_content(
        &self,
        id: Uuid,
        content: Option<text::Comment>,
    ) -> Result {
        let content = match content {
            Some(content) => content,
            None => read_from_stdin()?,
        };

        self.repo.set_comment_content(id, content).await?;

        Ok(())
    }

    pub async fn set_post_description(
        &self,
        id: Uuid,
        description: Option<text::Description>,
    ) -> Result {
        let description = match description {
            Some(description) => description,
            None => read_from_stdin()?,
        };

        self.repo.set_post_description(id, description).await?;

        Ok(())
    }

    pub async fn set_post_title(
        &self,
        id: Uuid,
        title: Option<text::PostTitle>,
    ) -> Result {
        let title = match title {
            Some(title) => title,
            None => read_from_stdin()?,
        };

        self.repo.set_post_title(id, title).await?;

        Ok(())
    }

    pub async fn set_tag_description(
        &self,
        id: Uuid,
        description: Option<text::Description>,
    ) -> Result {
        let description = match description {
            Some(description) => description,
            None => read_from_stdin()?,
        };

        self.repo.set_tag_description(id, description).await?;

        Ok(())
    }

    pub async fn set_tag_name(&self, id: Uuid, name: text::Name) -> Result {
        self.print(self.repo.set_tag_name(id, name).await?)
    }

    pub async fn set_user_description(
        &self,
        description: Option<text::Description>,
    ) -> Result {
        let description = match description {
            Some(description) => description,
            None => read_from_stdin()?,
        };

        self.repo.set_user_description(description).await?;

        Ok(())
    }

    pub async fn set_user_admin(&self, user: Uuid, admin: bool) -> Result {
        if admin {
            self.repo.grant_admin(user).await?;
        } else {
            self.repo.revoke_admin(user).await?;
        }

        Ok(())
    }

    pub async fn set_user_email(&self, email: text::Email) -> Result {
        self.repo.set_user_email(email).await?;
        Ok(())
    }

    pub async fn set_user_name(&self, name: text::Name) -> Result {
        self.print(self.repo.set_user_name(name).await?)
    }

    pub async fn set_user_password(&self) -> Result {
        let password = prompt_password("Password: ")?
            .try_into()
            .map_err(|err| format!("{err}"))?;
        self.repo.set_user_password(password).await?;
        Ok(())
    }

    pub async fn sign_out(&self) -> Result {
        self.repo.sign_out().await?;
        Ok(())
    }

    pub async fn sign_up(
        &self,
        username: text::Name,
        email: text::Email,
    ) -> crate::Result<String> {
        let password = prompt_password("Password: ")?
            .try_into()
            .map_err(|err| format!("{err}"))?;

        let info = SignUp {
            username,
            email,
            password,
        };

        Ok(self.repo.sign_up(&info).await?)
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

fn read_from_stdin<T>() -> crate::Result<T>
where
    T: FromStr<Err = text::Error>,
{
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer)?;

    Ok(buffer.parse().map_err(|err: text::Error| err.to_string())?)
}
