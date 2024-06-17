use crate::{Repo, Result};

use minty::{ProfileQuery, SearchResult, UserPreview};

pub struct Users<'a> {
    repo: &'a Repo,
}

impl<'a> Users<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn find(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<UserPreview>> {
        let results = self
            .repo
            .search
            .find_entities(&self.repo.search.indices.user, query)
            .await?;

        let users = self
            .repo
            .cache
            .users()
            .get_multiple(&results.hits)
            .await?
            .into_iter()
            .filter_map(|user| user.preview())
            .collect();

        Ok(SearchResult {
            total: results.total,
            hits: users,
        })
    }
}
