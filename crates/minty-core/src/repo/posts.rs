use super::{Post, Repo};

use crate::{db, Error, Result};

use minty::{
    PostParts, PostPreview, PostQuery, SearchResult, Uuid, Visibility,
};

pub struct Posts<'a> {
    repo: &'a Repo,
}

impl<'a> Posts<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn add(&self, user: Uuid, parts: &PostParts) -> Result<Post> {
        let mut tx = self.repo.database.begin().await?;

        let post = tx
            .create_post(
                user,
                parts.title.as_ref().map(|t| t.as_ref()).unwrap_or(""),
                parts.description.as_ref().map(|d| d.as_ref()).unwrap_or(""),
                parts.visibility.map(db::Visibility::from_minty),
                parts.objects.as_deref().unwrap_or(&[]),
                parts.posts.as_deref().unwrap_or(&[]),
                parts.tags.as_deref().unwrap_or(&[]),
            )
            .await?;

        self.repo.search.add_post(&post.search()).await?;

        tx.commit().await?;

        let post = self.repo.cache.posts().insert(post).await?;

        Ok(Post::new(self.repo, post))
    }

    pub async fn find(
        &self,
        user_id: Option<Uuid>,
        mut query: PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        if user_id.is_none() && query.visibility != Visibility::Public {
            return Err(Error::Unauthenticated(None));
        }

        if query.visibility == Visibility::Draft {
            query.poster = user_id;
        }

        let SearchResult { total, hits } =
            self.repo.search.find_posts(&query).await?;

        let hits = self.repo.cache.posts().previews(&hits).await?;

        Ok(SearchResult { total, hits })
    }
}
