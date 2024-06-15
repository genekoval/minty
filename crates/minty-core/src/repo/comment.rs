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

    pub async fn delete(&self, recursive: bool) -> Result<()> {
        self.repo
            .database
            .delete_comment(self.id, recursive)
            .await?
            .found("comment", self.id)?;

        self.repo.cache.comments().delete(self.id, recursive);

        Ok(())
    }

    pub async fn get(&self) -> Result<minty::Comment> {
        self.repo.cache.comments().get(self.id).await
    }

    pub async fn reply(
        &self,
        user_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        let user = self
            .repo
            .cache
            .users()
            .get(user_id)
            .await?
            .found("user", user_id)?;

        let comment = self
            .repo
            .database
            .create_reply(user_id, self.id, content.as_ref())
            .await?
            .found("comment", self.id)?;

        Ok(self.repo.cache.comments().reply(self.id, comment, user))
    }

    pub async fn set_content(&self, content: text::Comment) -> Result<String> {
        let content: String = content.into();

        self.repo
            .database
            .update_comment(self.id, &content)
            .await?
            .found("comment", self.id)?;

        self.repo.cache.comments().update(self.id, &content);

        Ok(content)
    }
}
