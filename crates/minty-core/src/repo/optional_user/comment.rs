use crate::{Repo, Result};

use minty::Uuid;

pub struct Comment<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Comment<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn get(&self) -> Result<minty::Comment> {
        self.repo.cache.comments().get(self.id).await
    }
}
