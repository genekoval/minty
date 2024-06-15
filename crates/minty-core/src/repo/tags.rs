use super::{Repo, Tag};

use crate::Result;

use minty::{text::Name, ProfileQuery, SearchResult, TagPreview, Uuid};

pub struct Tags<'a> {
    repo: &'a Repo,
}

impl<'a> Tags<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn add(&self, name: Name, creator: Uuid) -> Result<Tag> {
        let name = name.as_ref();
        let mut tx = self.repo.database.begin().await?;

        let tag = tx.create_tag(name, creator).await?;
        self.repo.search.add_tag_alias(tag.id, name).await?;

        tx.commit().await?;

        let tag = self.repo.cache.tags().insert(tag).await;
        Ok(Tag::new(self.repo, tag))
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
