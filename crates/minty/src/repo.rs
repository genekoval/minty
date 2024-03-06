use crate::{model::*, Result};

#[allow(async_fn_in_trait)]
pub trait Repo {
    fn new(url: &Url) -> Self;

    fn url(&self) -> &Url;

    async fn about(&self) -> Result<About>;

    async fn add_comment(
        &self,
        post_id: Uuid,
        content: &str,
    ) -> Result<CommentData>;

    async fn add_post_tag(&self, post_id: Uuid, tag_id: Uuid) -> Result<()>;

    async fn add_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()>;

    async fn add_reply(
        &self,
        parent_id: Uuid,
        content: &str,
    ) -> Result<CommentData>;

    async fn add_tag(&self, name: &str) -> Result<Uuid>;

    async fn add_tag_alias(&self, tag_id: Uuid, alias: &str)
        -> Result<TagName>;

    async fn add_tag_source(&self, tag_id: Uuid, url: &Url) -> Result<Source>;

    async fn append_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime>;

    async fn create_post(&self, post_id: Uuid) -> Result<()>;

    async fn create_post_draft(&self) -> Result<Uuid>;

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
    ) -> Result<TagName>;

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

    async fn get_comment(&self, id: Uuid) -> Result<Comment>;

    async fn get_comments(&self, post_id: Uuid) -> Result<Vec<CommentData>>;

    async fn get_object(&self, id: Uuid) -> Result<Object>;

    async fn get_post(&self, id: Uuid) -> Result<Post>;

    async fn get_posts(
        &self,
        query: &PostQuery,
    ) -> Result<SearchResult<PostPreview>>;

    async fn get_tag(&self, id: Uuid) -> Result<Tag>;

    async fn get_tags(
        &self,
        query: &TagQuery,
    ) -> Result<SearchResult<TagPreview>>;

    async fn insert_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
        destination: Uuid,
    ) -> Result<DateTime>;

    async fn set_comment_content(
        &self,
        comment_id: Uuid,
        content: &str,
    ) -> Result<String>;

    async fn set_post_description(
        &self,
        post_id: Uuid,
        description: &str,
    ) -> Result<Modification<String>>;

    async fn set_post_title(
        &self,
        post_id: Uuid,
        title: &str,
    ) -> Result<Modification<String>>;

    async fn set_tag_description(
        &self,
        tag_id: Uuid,
        description: &str,
    ) -> Result<String>;

    async fn set_tag_name(
        &self,
        tag_id: Uuid,
        new_name: &str,
    ) -> Result<TagName>;
}
