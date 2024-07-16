use super::Tag;

use crate::{cache::User, Cached, Repo, Result};

use minty::text::Name;
use std::sync::Arc;

pub struct Tags<'a> {
    repo: &'a Repo,
    user: Arc<Cached<User>>,
}

impl<'a> Tags<'a> {
    pub(super) fn new(repo: &'a Repo, user: Arc<Cached<User>>) -> Self {
        Self { repo, user }
    }

    pub async fn add(self, name: Name) -> Result<Tag<'a>> {
        let name = name.as_ref();
        let mut tx = self.repo.database.begin().await?;

        let tag = tx.create_tag(name, self.user.id).await?;
        self.repo.search.add_tag_alias(tag.id, name).await?;
        let tag = self.repo.cache.tags().insert(tag, self.user);

        tx.commit().await?;
        Ok(Tag::new(self.repo, tag))
    }
}
