use crate::{error::Found, Repo, Result};

use bytes::Bytes;
use futures::Stream;
use minty::{ObjectSummary, Uuid};
use std::io;

pub struct Object<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Object<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn get(&self) -> Result<minty::Object> {
        let cache = &self.repo.cache;
        cache
            .objects()
            .get(self.id)
            .await?
            .found("object", self.id)?
            .model(cache)
            .await
    }

    pub async fn get_data(
        &self,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)> {
        self.repo.bucket.get_object_stream(self.id).await
    }
}
