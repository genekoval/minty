use super::Repo;

use crate::{preview, Result};

use log::error;
use minty::ObjectPreview;

pub struct Objects<'a> {
    repo: &'a Repo,
}

impl<'a> Objects<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub(super) async fn add(
        &self,
        object: fstore::Object,
    ) -> Result<ObjectPreview> {
        let result =
            preview::generate_preview(&self.repo.bucket, &object).await;
        let preview = result.as_ref().ok().cloned().flatten();

        self.repo
            .database
            .create_object(object.id, preview, None)
            .await?;

        if let Err(preview_error) = result {
            if let Err(err) = self
                .repo
                .database
                .create_object_preview_error(object.id, &preview_error)
                .await
            {
                error!(
                    "Failed to write object preview error to database: {err}; \
                    error for object '{}': {preview_error}",
                    object.id
                );
            }
        }

        Ok(ObjectPreview {
            id: object.id,
            preview_id: preview,
            r#type: object.r#type,
            subtype: object.subtype,
        })
    }
}
