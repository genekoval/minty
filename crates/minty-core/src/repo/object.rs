use super::Repo;

use crate::{preview, Error, Result};

use minty::Uuid;

pub struct Object<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Object<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
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
