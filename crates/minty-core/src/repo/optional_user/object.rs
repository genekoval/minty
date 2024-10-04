use crate::{error::Found, Cached, Repo, Result, User};

use bytes::Bytes;
use fstore::http::{ProxyMethod, ProxyResponse, Range};
use futures::Stream;
use minty::Uuid;
use std::{io, ops::RangeBounds, sync::Arc};

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
        range: impl RangeBounds<u64>,
    ) -> Result<impl Stream<Item = io::Result<Bytes>>> {
        self.repo.bucket.get_object_stream(self.id, range).await
    }

    pub async fn proxy(
        &self,
        method: ProxyMethod,
        range: Option<Range>,
    ) -> Result<ProxyResponse<impl Stream<Item = io::Result<Bytes>>>> {
        self.repo.bucket.proxy(self.id, method, range).await
    }
}
