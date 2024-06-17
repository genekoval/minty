use crate::{Repo, Result};

use bytes::Bytes;
use futures::TryStream;
use minty::ObjectPreview;

pub struct Objects<'a> {
    repo: &'a Repo,
}

impl<'a> Objects<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn upload<S>(&self, stream: S) -> Result<ObjectPreview>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        let object = self.repo.bucket.add_object_stream(stream).await?;
        self.repo.objects().add(object).await
    }
}
