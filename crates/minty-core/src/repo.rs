use crate::error::Result;

use minty::model::*;

#[derive(Debug)]
pub struct Repo {}

impl Repo {
    pub async fn add_comment(
        post_id: Uuid,
        content: &str,
    ) -> Result<CommentData> {
        todo!()
    }

    pub async fn add_post_objects(
        post_id: Uuid,
        objects: &[Uuid],
        destination: Option<Uuid>,
    ) -> Result<DateTime> {
        todo!()
    }

    pub async fn add_post_tag(post_id: Uuid, tag_id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn add_related_post(post_id: Uuid, related: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn add_reply(
        parent_id: Uuid,
        content: &str,
    ) -> Result<CommentData> {
        todo!()
    }

    pub async fn add_tag(name: &str) -> Result<Uuid> {
        todo!()
    }

    pub async fn add_tag_alias(tag_id: Uuid, alias: &str) -> Result<TagName> {
        todo!()
    }

    pub async fn add_tag_source(tag_id: Uuid, url: &Url) -> Result<Source> {
        todo!()
    }

    pub async fn create_post(post_id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn create_post_draft() -> Result<Uuid> {
        todo!()
    }

    pub async fn delete_comment(id: Uuid, recursive: bool) -> Result<bool> {
        todo!()
    }

    pub async fn delete_post(id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn delete_post_objects(
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime> {
        todo!()
    }

    pub async fn delete_post_tag(post_id: Uuid, tag_id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn delete_related_post(
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        todo!()
    }

    pub async fn delete_tag(id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn delete_tag_alias(
        tag_id: Uuid,
        alias: &str,
    ) -> Result<TagName> {
        todo!()
    }

    pub async fn delete_tag_source(tag_id: Uuid, source_id: i64) -> Result<()> {
        todo!()
    }

    pub async fn get_comment(id: Uuid) -> Result<Comment> {
        todo!()
    }

    pub async fn get_comments(post_id: Uuid) -> Result<Vec<CommentData>> {
        todo!()
    }

    pub async fn get_object(id: Uuid) -> Result<Object> {
        todo!()
    }

    pub async fn get_object_preview_errors() -> Vec<ObjectError> {
        todo!()
    }

    pub async fn get_post(id: Uuid) -> Result<Post> {
        todo!()
    }

    pub async fn get_posts(
        query: &PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        todo!()
    }

    pub async fn get_tag(id: Uuid) -> Result<Tag> {
        todo!()
    }

    pub async fn get_tags(
        query: &TagQuery,
    ) -> Result<SearchResult<TagPreview>> {
        todo!()
    }

    pub async fn prune() -> Result<()> {
        todo!()
    }

    pub async fn regenerate_preview(object_id: Uuid) -> Result<Option<Uuid>> {
        todo!()
    }

    pub async fn set_comment_content(
        comment_id: Uuid,
        content: &str,
    ) -> Result<String> {
        todo!()
    }

    pub async fn set_post_description(
        post_id: Uuid,
        description: &str,
    ) -> Result<Modification<String>> {
        todo!()
    }

    pub async fn set_post_title(
        post_id: Uuid,
        title: &str,
    ) -> Result<Modification<String>> {
        todo!()
    }

    pub async fn set_tag_description(
        tag_id: Uuid,
        description: &str,
    ) -> Result<String> {
        todo!()
    }

    pub async fn set_tag_name(tag_id: Uuid, new_name: &str) -> Result<TagName> {
        todo!()
    }
}
