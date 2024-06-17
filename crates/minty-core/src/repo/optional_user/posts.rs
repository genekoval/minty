use crate::{cache::User, Cached, Error, Repo, Result};

use minty::{PostPreview, PostQuery, SearchResult, Visibility};
use std::sync::Arc;

pub struct Posts<'a> {
    repo: &'a Repo,
    user: Option<Arc<Cached<User>>>,
}

impl<'a> Posts<'a> {
    pub(super) fn new(repo: &'a Repo, user: Option<Arc<Cached<User>>>) -> Self {
        Self { repo, user }
    }

    pub async fn find(
        &self,
        mut query: PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        if query.visibility == Visibility::Draft {
            if let Some(user) = self.user.as_ref() {
                query.poster = Some(user.id);
            } else {
                return Err(Error::Unauthenticated(None));
            }
        }

        let SearchResult { total, hits } =
            self.repo.search.find_posts(&query).await?;

        let hits = self.repo.cache.posts().previews(&hits).await?;

        Ok(SearchResult { total, hits })
    }
}
