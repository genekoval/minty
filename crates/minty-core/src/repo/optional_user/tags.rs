use crate::{Repo, Result};

use minty::{ProfileQuery, SearchResult, TagPreview, Uuid};

pub struct Tags<'a> {
    repo: &'a Repo,
}

impl<'a> Tags<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn get(&self, tags: &[Uuid]) -> Result<Vec<TagPreview>> {
        self.repo.cache.tags().previews(tags).await
    }

    pub async fn find(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<TagPreview>> {
        let SearchResult { total, hits } = self
            .repo
            .search
            .find_entities(&self.repo.search.indices.tag, query)
            .await?;

        let hits = self.repo.cache.tags().previews(&hits).await?;

        Ok(SearchResult { total, hits })
    }
}
