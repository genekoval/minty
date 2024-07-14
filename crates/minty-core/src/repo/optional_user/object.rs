use crate::{error::Found, Cached, Repo, Result, User};

use bytes::Bytes;
use futures::Stream;
use minty::{ObjectSummary, Uuid};
use std::{io, sync::Arc};

pub struct Object<'a> {
    repo: &'a Repo,
    user: Option<Arc<Cached<User>>>,
    id: Uuid,
}

impl<'a> Object<'a> {
    pub(super) fn new(
        repo: &'a Repo,
        user: Option<Arc<Cached<User>>>,
        id: Uuid,
    ) -> Self {
        Self { repo, user, id }
    }

    pub async fn get(&self) -> Result<minty::Object> {
        let cache = &self.repo.cache;
        cache
            .objects()
            .get(self.id)
            .await?
            .found("object", self.id)?
            .model(cache, self.user.as_ref())
            .await
    }

    pub async fn get_data(
        &self,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)> {
        self.repo.bucket.get_object_stream(self.id).await
    }
}
