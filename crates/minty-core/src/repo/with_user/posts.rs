use super::Post;

use crate::{cache::User, db, Cached, Repo, Result};

use minty::PostParts;
use std::sync::Arc;

pub struct Posts<'a> {
    repo: &'a Repo,
    user: Arc<Cached<User>>,
}

impl<'a> Posts<'a> {
    pub(super) fn new(repo: &'a Repo, user: Arc<Cached<User>>) -> Self {
        Self { repo, user }
    }

    pub async fn add(self, parts: &PostParts) -> Result<Post<'a>> {
        let mut tx = self.repo.database.begin().await?;

        let post = tx
            .create_post(
                self.user.id,
                parts.title.as_ref().map(|t| t.as_ref()).unwrap_or(""),
                parts.description.as_ref().map(|d| d.as_ref()).unwrap_or(""),
                parts.visibility.map(db::Visibility::from_minty),
                parts.objects.as_deref().unwrap_or(&[]),
                parts.posts.as_deref().unwrap_or(&[]),
                parts.tags.as_deref().unwrap_or(&[]),
            )
            .await?;

        self.repo.search.add_post(&post.search()).await?;

        let post = self.repo.cache.posts().insert(post).await?;

        tx.commit().await?;
        Ok(Post::new(self.repo, self.user, post))
    }
}
