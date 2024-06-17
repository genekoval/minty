use crate::{Repo, Result};

use minty::ObjectError;

pub struct Objects<'a> {
    repo: &'a Repo,
}

impl<'a> Objects<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn get_preview_errors(&self) -> Result<Vec<ObjectError>> {
        Ok(self
            .repo
            .database
            .read_object_preview_errors()
            .await?
            .into_iter()
            .map(|e| e.into())
            .collect())
    }
}
