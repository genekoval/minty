use crate::{Repo, Result};

use minty::{ProfileQuery, SearchResult, TagPreview};

pub struct Tags<'a> {
    repo: &'a Repo,
}

impl<'a> Tags<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
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

        let hits = self
            .repo
            .cache
            .tags()
            .get_multiple(&hits)
            .await?
            .into_iter()
            .filter_map(|tag| tag.preview())
            .collect();

        Ok(SearchResult { total, hits })
    }
}
