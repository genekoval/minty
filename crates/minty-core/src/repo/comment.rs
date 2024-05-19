use super::Repo;

use crate::{error::Found, Result};

use minty::{text, CommentData, Uuid};

pub struct Comment<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Comment<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn delete(&self, recursive: bool) -> Result<bool> {
        Ok(self
            .repo
            .database
            .delete_comment(self.id, recursive)
            .await?)
    }

    pub async fn get(&self) -> Result<minty::Comment> {
        Ok(self
            .repo
            .database
            .read_comment(self.id)
            .await?
            .found("comment", self.id)?
            .into())
    }

    pub async fn reply(
        &self,
        user_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        Ok(self
            .repo
            .database
            .create_reply(user_id, self.id, content.as_ref())
            .await?
            .found("comment", self.id)?
            .into())
    }

    pub async fn set_content(&self, content: text::Comment) -> Result<String> {
        self.repo
            .database
            .update_comment(self.id, content.as_ref())
            .await?
            .found("comment", self.id)?;

        Ok(content.into())
    }
}
