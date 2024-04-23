pub mod query;

mod client;

use client::Client;

use crate::{model::*, text, Result};

use bytes::Bytes;
use futures_core::{Stream, TryStream};
use std::{error::Error, io};

#[derive(Clone, Debug)]
pub struct Repo {
    client: Client,
}

impl crate::Repo for Repo {
    fn new(url: &Url) -> Self {
        Self {
            client: Client::new(url),
        }
    }

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
        S::Error: Into<Box<dyn Error + Send + Sync>>,
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

    async fn add_tag(&self, name: text::TagName) -> Result<Uuid> {
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
        alias: text::TagName,
    ) -> Result<TagName> {
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
            .serialize(url)
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
            .serialize(objects)
            .send()
            .await?
            .date_time()
            .await
    }

    async fn create_post(&self, parts: &PostParts) -> Result<Uuid> {
        self.client
            .post("post")
            .serialize(parts)
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
            .serialize(objects)
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
    ) -> Result<TagName> {
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
            .serialize(sources)
            .send()
            .await?;

        Ok(())
    }

    #[cfg(feature = "export")]
    async fn export(&self) -> Result<export::Data> {
        self.client.get("export").send().await?.deserialize().await
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
        query: &TagQuery,
    ) -> Result<SearchResult<TagPreview>> {
        let query: query::TagQuery = query.clone().into();
        self.client
            .get("tags")
            .query(&query)
            .send()
            .await?
            .deserialize()
            .await
    }

    async fn insert_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
        destination: Uuid,
    ) -> Result<DateTime> {
        self.client
            .post(format!("post/{post_id}/objects/{destination}"))
            .serialize(objects)
            .send()
            .await?
            .date_time()
            .await
    }

    async fn publish_post(&self, post_id: Uuid) -> Result<()> {
        self.client.put(format!("post/{post_id}")).send().await?;
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
        new_name: text::TagName,
    ) -> Result<TagName> {
        let query = query::SetTagName::main(true);

        self.client
            .put(format!("tag/{tag_id}/name/{new_name}"))
            .query(&query)
            .send()
            .await?
            .deserialize()
            .await
    }
}
