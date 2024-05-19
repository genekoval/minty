use std::io;

use super::Repo;

use crate::{error::Found, preview, Error, Result};

use bytes::Bytes;
use futures::Stream;
use minty::{ObjectSummary, Uuid};

pub struct Object<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Object<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn get(&self) -> Result<minty::Object> {
        let object = self
            .repo
            .database
            .read_object(self.id)
            .await?
            .found("object", self.id)?;

        let posts = self.repo.database.read_object_posts(self.id).await?;
        let metadata = self.repo.bucket.get_object(self.id).await?;

        Ok(minty::Object {
            id: self.id,
            hash: metadata.hash,
            size: metadata.size,
            r#type: metadata.r#type,
            subtype: metadata.subtype,
            added: metadata.added,
            preview_id: object.preview_id,
            posts: self.repo.posts().build(posts).await?,
        })
    }

    pub async fn get_data(
        &self,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)> {
        self.repo.bucket.get_object_stream(self.id).await
    }

    pub async fn regenerate_preview(&self) -> Result<Option<Uuid>> {
        let object = self.repo.bucket.get_object(self.id).await?;

        match preview::generate_preview(&self.repo.bucket, &object).await {
            Ok(preview) => {
                self.repo
                    .database
                    .update_object_preview(object.id, preview)
                    .await?;
                Ok(preview)
            }
            Err(message) => {
                self.repo
                    .database
                    .create_object_preview_error(object.id, &message)
                    .await?;
                Err(Error::Internal(message))
            }
        }
    }
}
