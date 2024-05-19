use super::Repo;

use crate::Result;

use minty::{text::Name, ProfileQuery, SearchResult, TagPreview, Uuid};

pub struct Tags<'a> {
    repo: &'a Repo,
}

impl<'a> Tags<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn add(&self, name: Name, creator: Uuid) -> Result<Uuid> {
        let name = name.as_ref();
        let mut tx = self.repo.database.begin().await?;

        let id = tx.create_tag(name, creator).await?.0;
        self.repo.search.add_tag_alias(id, name).await?;

        tx.commit().await?;
        Ok(id)
    }

    pub async fn find(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<TagPreview>> {
        let results = self
            .repo
            .search
            .find_entities(&self.repo.search.indices.tag, query)
            .await?;
        let tags = self
            .repo
            .database
            .read_tag_previews(&results.hits)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(SearchResult {
            total: results.total,
            hits: tags,
        })
    }
}
