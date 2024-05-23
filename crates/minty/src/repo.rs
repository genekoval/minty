use crate::{model::*, text, Result};

use bytes::Bytes;
use futures_core::{Stream, TryStream};
use std::{error::Error, io};

#[allow(async_fn_in_trait)]
pub trait Repo {
    fn new(url: &Url, session: Option<String>) -> Self;

    fn url(&self) -> &Url;

    async fn about(&self) -> Result<About>;

    async fn add_comment(
        &self,
        post_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData>;

    async fn add_object<S>(&self, stream: S) -> Result<ObjectPreview>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn Error + Send + Sync>>,
        Bytes: From<S::Ok>;

    async fn add_post_tag(&self, post_id: Uuid, tag_id: Uuid) -> Result<()>;

    async fn add_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()>;

    async fn add_reply(
        &self,
        parent_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData>;

    async fn add_tag(&self, name: text::Name) -> Result<Uuid>;

    async fn add_tag_alias(
        &self,
        tag_id: Uuid,
        alias: text::Name,
    ) -> Result<ProfileName>;

    async fn add_tag_source(&self, tag_id: Uuid, url: &Url) -> Result<Source>;

    async fn add_user_alias(&self, alias: text::Name) -> Result<ProfileName>;

    async fn add_user_source(&self, url: &Url) -> Result<Source>;

    async fn append_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime>;

    async fn authenticate(&self, login: &Login) -> Result<String>;

    async fn create_post(&self, parts: &PostParts) -> Result<Uuid>;

    async fn delete_comment(&self, id: Uuid, recursive: bool) -> Result<()>;

    async fn delete_post(&self, id: Uuid) -> Result<()>;

    async fn delete_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime>;

    async fn delete_post_tag(&self, post_id: Uuid, tag_id: Uuid) -> Result<()>;

    async fn delete_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()>;

    async fn delete_tag(&self, id: Uuid) -> Result<()>;

    async fn delete_tag_alias(
        &self,
        tag_id: Uuid,
        alias: &str,
    ) -> Result<ProfileName>;

    async fn delete_tag_source(
        &self,
        tag_id: Uuid,
        source_id: i64,
    ) -> Result<()>;

    async fn delete_tag_sources(
        &self,
        tag_id: Uuid,
        sources: &[String],
    ) -> Result<()>;

    async fn delete_user(&self) -> Result<()>;

    async fn delete_user_alias(&self, alias: &str) -> Result<ProfileName>;

    async fn delete_user_source(&self, source_id: i64) -> Result<()>;

    async fn delete_user_sources(&self, sources: &[String]) -> Result<()>;

    #[cfg(feature = "export")]
    async fn export(&self) -> Result<export::Data>;

    async fn get_authenticated_user(&self) -> Result<User>;

    async fn get_comment(&self, id: Uuid) -> Result<Comment>;

    async fn get_comments(&self, post_id: Uuid) -> Result<Vec<CommentData>>;

    async fn get_object(&self, id: Uuid) -> Result<Object>;

    async fn get_object_data(
        &self,
        id: Uuid,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)>;

    async fn get_post(&self, id: Uuid) -> Result<Post>;

    async fn get_posts(
        &self,
        query: &PostQuery,
    ) -> Result<SearchResult<PostPreview>>;

    async fn get_tag(&self, id: Uuid) -> Result<Tag>;

    async fn get_tags(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<TagPreview>>;

    async fn get_user(&self, id: Uuid) -> Result<User>;

    async fn get_users(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<UserPreview>>;

    async fn grant_admin(&self, user_id: Uuid) -> Result<()>;

    async fn insert_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
        destination: Uuid,
    ) -> Result<DateTime>;

    async fn publish_post(&self, post_id: Uuid) -> Result<()>;

    async fn revoke_admin(&self, user_id: Uuid) -> Result<()>;

    async fn set_comment_content(
        &self,
        comment_id: Uuid,
        content: text::Comment,
    ) -> Result<String>;

    async fn set_post_description(
        &self,
        post_id: Uuid,
        description: text::Description,
    ) -> Result<Modification<String>>;

    async fn set_post_title(
        &self,
        post_id: Uuid,
        title: text::PostTitle,
    ) -> Result<Modification<String>>;

    async fn set_tag_description(
        &self,
        tag_id: Uuid,
        description: text::Description,
    ) -> Result<String>;

    async fn set_tag_name(
        &self,
        tag_id: Uuid,
        new_name: text::Name,
    ) -> Result<ProfileName>;

    async fn set_user_description(
        &self,
        description: text::Description,
    ) -> Result<String>;

    async fn set_user_email(&self, email: text::Email) -> Result<()>;

    async fn set_user_name(&self, new_name: text::Name) -> Result<ProfileName>;

    async fn set_user_password(&self, password: text::Password) -> Result<()>;

    async fn sign_out(&self) -> Result<()>;

    async fn sign_up(&self, info: &SignUp) -> Result<String>;
}
