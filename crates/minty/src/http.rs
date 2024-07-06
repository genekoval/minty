pub mod cookie;
pub mod query;

mod client;

use client::Client;
use cookie::{CookieFile, Jar};

use crate::{model::*, text, Error, Result};

use bytes::Bytes;
use futures_core::{Stream, TryStream};
use reqwest::ClientBuilder;
use std::{error::Error as StdError, io, sync::Arc};

#[derive(Clone, Debug)]
pub enum Credentials {
    None,
    Cookies,
    CookieJar(Arc<Jar>),
    CookieFile(Arc<CookieFile>),
}

#[derive(Debug)]
pub struct RepoBuilder {
    builder: ClientBuilder,
    url: Url,
}

impl RepoBuilder {
    fn new(url: Url) -> Self {
        Self {
            builder: ClientBuilder::new(),
            url,
        }
    }

    pub fn build(self) -> Result<Repo> {
        let client = self.builder.build().map_err(|err| {
            Error::other(format!("failed to build HTTP client: {err}"))
        })?;

        Ok(Repo {
            client: Client::new(client, self.url),
        })
    }

    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.builder = match credentials {
            Credentials::None => self.builder.cookie_store(false),
            Credentials::Cookies => self.builder.cookie_store(true),
            Credentials::CookieJar(jar) => self.builder.cookie_provider(jar),
            Credentials::CookieFile(file) => self.builder.cookie_provider(file),
        };

        self
    }
}

#[derive(Clone, Debug)]
pub struct Repo {
    client: Client,
}

impl Repo {
    pub fn new(url: Url) -> Self {
        Self {
            client: Client::new(reqwest::Client::new(), url),
        }
    }

    pub fn build(url: Url) -> RepoBuilder {
        RepoBuilder::new(url)
    }
}

impl crate::Repo for Repo {
    fn url(&self) -> &Url {
        self.client.url()
    }

    async fn about(&self) -> Result<About> {
        self.client.get("/").send().await?.deserialize().await
    }

    async fn add_comment(
        &self,
        post_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        self.client
            .post(format!("comments/{post_id}"))
            .text(content.into())
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn add_object<S>(&self, stream: S) -> Result<ObjectPreview>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn StdError + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        self.client
            .post("object")
            .stream(stream)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn add_post_tag(&self, post_id: Uuid, tag_id: Uuid) -> Result<()> {
        self.client
            .put(format!("post/{post_id}/tag/{tag_id}"))
            .send()
            .await?;

        Ok(())
    }

    async fn add_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        self.client
            .put(format!("post/{post_id}/related/{related}"))
            .send()
            .await?;

        Ok(())
    }

    async fn add_reply(
        &self,
        parent_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        self.client
            .post(format!("comment/{parent_id}"))
            .text(content.into())
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn add_tag(&self, name: text::Name) -> Result<Uuid> {
        self.client
            .post(format!("tag/{name}"))
            .send()
            .await?
            .uuid()
            .await
    }

    async fn add_tag_alias(
        &self,
        tag_id: Uuid,
        alias: text::Name,
    ) -> Result<ProfileName> {
        self.client
            .put(format!("tag/{tag_id}/name/{alias}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn add_tag_source(&self, tag_id: Uuid, url: &Url) -> Result<Source> {
        self.client
            .post(format!("tag/{tag_id}/source"))
            .json(url)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn add_user_alias(&self, alias: text::Name) -> Result<ProfileName> {
        self.client
            .put(format!("user/name/{alias}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn add_user_source(&self, url: &Url) -> Result<Source> {
        self.client
            .post("user/source")
            .json(url)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn append_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime> {
        self.client
            .post(format!("post/{post_id}/objects"))
            .json(objects)
            .send()
            .await?
            .date_time()
            .await
    }

    async fn authenticate(&self, login: &Login) -> Result<String> {
        self.client
            .post("user/session")
            .json(login)
            .send()
            .await?
            .text()
            .await
    }

    async fn create_post(&self, parts: &PostParts) -> Result<Uuid> {
        self.client
            .post("post")
            .json(parts)
            .send()
            .await?
            .uuid()
            .await
    }

    async fn delete_comment(&self, id: Uuid, recursive: bool) -> Result<()> {
        self.client
            .delete(format!("comment/{id}"))
            .query(&[("recursive", recursive)])
            .send()
            .await?;

        Ok(())
    }

    async fn delete_post(&self, id: Uuid) -> Result<()> {
        self.client.delete(format!("post/{id}")).send().await?;
        Ok(())
    }

    async fn delete_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime> {
        self.client
            .delete(format!("post/{post_id}/objects"))
            .json(objects)
            .send()
            .await?
            .date_time()
            .await
    }

    async fn delete_post_tag(&self, post_id: Uuid, tag_id: Uuid) -> Result<()> {
        self.client
            .delete(format!("post/{post_id}/tag/{tag_id}"))
            .send()
            .await?;

        Ok(())
    }

    async fn delete_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        self.client
            .delete(format!("post/{post_id}/related/{related}"))
            .send()
            .await?;

        Ok(())
    }

    async fn delete_tag(&self, id: Uuid) -> Result<()> {
        self.client.delete(format!("tag/{id}")).send().await?;
        Ok(())
    }

    async fn delete_tag_alias(
        &self,
        tag_id: Uuid,
        alias: &str,
    ) -> Result<ProfileName> {
        self.client
            .delete(format!("tag/{tag_id}/name/{alias}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn delete_tag_source(
        &self,
        tag_id: Uuid,
        source_id: i64,
    ) -> Result<()> {
        self.client
            .delete(format!("tag/{tag_id}/source/{source_id}"))
            .send()
            .await?;

        Ok(())
    }

    async fn delete_tag_sources(
        &self,
        tag_id: Uuid,
        sources: &[String],
    ) -> Result<()> {
        self.client
            .delete(format!("tag/{tag_id}/source"))
            .json(sources)
            .send()
            .await?;

        Ok(())
    }

    async fn delete_user(&self) -> Result<()> {
        self.client.delete("user").send().await?;
        Ok(())
    }

    async fn delete_user_alias(&self, alias: &str) -> Result<ProfileName> {
        self.client
            .delete(format!("user/name/{alias}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn delete_user_source(&self, source_id: i64) -> Result<()> {
        self.client
            .delete(format!("user/source/{source_id}"))
            .send()
            .await?;

        Ok(())
    }

    async fn delete_user_sources(&self, sources: &[String]) -> Result<()> {
        self.client
            .delete("user/source")
            .json(sources)
            .send()
            .await?;

        Ok(())
    }

    #[cfg(feature = "export")]
    async fn export(&self) -> Result<export::Data> {
        self.client.get("export").send().await?.deserialize().await
    }

    async fn get_authenticated_user(&self) -> Result<User> {
        self.client.get("user").send().await?.deserialize().await
    }

    async fn get_comment(&self, id: Uuid) -> Result<Comment> {
        self.client
            .get(format!("comment/{id}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_comments(&self, post_id: Uuid) -> Result<Vec<CommentData>> {
        self.client
            .get(format!("comments/{post_id}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_invitation(&self) -> Result<String> {
        self.client.get("invitation").send().await?.text().await
    }

    async fn get_inviter(&self, invitation: &str) -> Result<User> {
        self.client
            .get(format!("invitation/{invitation}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_object(&self, id: Uuid) -> Result<Object> {
        self.client
            .get(format!("object/{id}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_object_data(
        &self,
        id: Uuid,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)> {
        self.client
            .get(format!("object/{id}/data"))
            .send()
            .await?
            .object()
    }

    async fn get_object_preview_errors(&self) -> Result<Vec<ObjectError>> {
        self.client
            .get("objects/errors")
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_post(&self, id: Uuid) -> Result<Post> {
        self.client
            .get(format!("post/{id}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_posts(
        &self,
        query: &PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        let query: query::PostQuery = query.clone().into();
        self.client
            .get("posts")
            .query(&query)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_tag(&self, id: Uuid) -> Result<Tag> {
        self.client
            .get(format!("tag/{id}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_tags(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<TagPreview>> {
        let query: query::ProfileQuery = query.clone().into();
        self.client
            .get("tags")
            .query(&query)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_user(&self, id: Uuid) -> Result<User> {
        self.client
            .get(format!("user/{id}"))
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn get_users(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<UserPreview>> {
        let query: query::ProfileQuery = query.clone().into();
        self.client
            .get("users")
            .query(&query)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn grant_admin(&self, user_id: Uuid) -> Result<()> {
        self.client
            .put(format!("user/{user_id}/admin"))
            .send()
            .await?;
        Ok(())
    }

    async fn insert_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
        destination: Uuid,
    ) -> Result<DateTime> {
        self.client
            .post(format!("post/{post_id}/objects/{destination}"))
            .json(objects)
            .send()
            .await?
            .date_time()
            .await
    }

    async fn publish_post(&self, post_id: Uuid) -> Result<()> {
        self.client.put(format!("post/{post_id}")).send().await?;
        Ok(())
    }

    async fn revoke_admin(&self, user_id: Uuid) -> Result<()> {
        self.client
            .delete(format!("user/{user_id}/admin"))
            .send()
            .await?;
        Ok(())
    }

    async fn set_comment_content(
        &self,
        comment_id: Uuid,
        content: text::Comment,
    ) -> Result<String> {
        self.client
            .put(format!("comment/{comment_id}"))
            .text(content.into())
            .send()
            .await?
            .text()
            .await
    }

    async fn set_post_description(
        &self,
        post_id: Uuid,
        description: text::Description,
    ) -> Result<Modification<String>> {
        self.client
            .put(format!("post/{post_id}/description"))
            .text(description.into())
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn set_post_title(
        &self,
        post_id: Uuid,
        title: text::PostTitle,
    ) -> Result<Modification<String>> {
        self.client
            .put(format!("post/{post_id}/title"))
            .text(title.into())
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn set_tag_description(
        &self,
        tag_id: Uuid,
        description: text::Description,
    ) -> Result<String> {
        self.client
            .put(format!("tag/{tag_id}/description"))
            .text(description.into())
            .send()
            .await?
            .text()
            .await
    }

    async fn set_tag_name(
        &self,
        tag_id: Uuid,
        new_name: text::Name,
    ) -> Result<ProfileName> {
        let query = query::SetProfileName::main(true);

        self.client
            .put(format!("tag/{tag_id}/name/{new_name}"))
            .query(&query)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn set_user_description(
        &self,
        description: text::Description,
    ) -> Result<String> {
        self.client
            .put("user/description")
            .text(description.into())
            .send()
            .await?
            .text()
            .await
    }

    async fn set_user_email(&self, email: text::Email) -> Result<()> {
        self.client
            .put("user/email")
            .text(email.into())
            .send()
            .await?;
        Ok(())
    }

    async fn set_user_name(&self, new_name: text::Name) -> Result<ProfileName> {
        let query = query::SetProfileName::main(true);

        self.client
            .put(format!("user/name/{new_name}"))
            .query(&query)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn set_user_password(&self, password: text::Password) -> Result<()> {
        self.client
            .put("user/password")
            .text(password.into())
            .send()
            .await?;
        Ok(())
    }

    async fn sign_out(&self) -> Result<()> {
        self.client.delete("user/session").send().await?;
        Ok(())
    }

    async fn sign_up(
        &self,
        info: &SignUp,
        invitation: Option<String>,
    ) -> Result<String> {
        let query = query::SignUp { invitation };

        self.client
            .post("signup")
            .query(&query)
            .form(info)
            .send()
            .await?
            .text()
            .await
    }
}
