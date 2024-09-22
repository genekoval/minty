use crate::{
    cache::{self, PostComments, User},
    error::Found,
    Cached, Repo, Result,
};

use minty::{CommentData, ObjectPreview, PostPreview};
use std::sync::Arc;

pub struct Post<'a> {
    repo: &'a Repo,
    user: Option<Arc<Cached<User>>>,
    post: Arc<Cached<cache::Post>>,
}

impl<'a> Post<'a> {
    pub(super) fn new(
        repo: &'a Repo,
        user: Option<Arc<Cached<User>>>,
        post: Arc<Cached<cache::Post>>,
    ) -> Result<Self> {
        post.can_view(user.as_ref())?;

        Ok(Self { repo, user, post })
    }

    async fn comments<F, R>(&self, f: F) -> Result<R>
    where
        F: Fn(PostComments<'_>) -> R,
    {
        self.post.comments(&self.post, &self.repo.cache, f).await
    }

    pub async fn get(&self) -> Result<minty::Post> {
        self.post
            .model(&self.repo.cache, self.user.as_ref())
            .await?
            .found("post", self.post.id)
    }

    pub async fn get_comments(&self) -> Result<Vec<CommentData>> {
        self.comments(|comments| comments.get_all()).await
    }

    pub fn get_objects(&self) -> Result<Vec<ObjectPreview>> {
        self.post.objects().found("post", self.post.id)
    }

    pub fn preview(&self) -> Result<PostPreview> {
        self.post
            .preview(self.user.as_ref())
            .found("post", self.post.id)
    }
}
