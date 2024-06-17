use crate::{cache::User, error::Found, Cached, Repo, Result};

use minty::{text, CommentData, Uuid};
use std::sync::Arc;

pub struct Comment<'a> {
    repo: &'a Repo,
    user: Arc<Cached<User>>,
    id: Uuid,
}

impl<'a> Comment<'a> {
    pub(super) fn new(
        repo: &'a Repo,
        user: Arc<Cached<User>>,
        id: Uuid,
    ) -> Self {
        Self { repo, user, id }
    }

    pub async fn delete(&self, recursive: bool) -> Result<()> {
        if recursive {
            self.user.deny_permission()?;
        } else {
            self.repo
                .cache
                .comments()
                .can_edit(self.id, &self.user)
                .await?;
        }

        self.repo
            .database
            .delete_comment(self.id, recursive)
            .await?
            .found("comment", self.id)?;

        self.repo.cache.comments().delete(self.id, recursive);

        Ok(())
    }

    pub async fn reply(self, content: text::Comment) -> Result<CommentData> {
        let comment = self
            .repo
            .database
            .create_reply(self.user.id, self.id, content.as_ref())
            .await?
            .found("comment", self.id)?;

        Ok(self
            .repo
            .cache
            .comments()
            .reply(self.id, comment, self.user))
    }

    pub async fn set_content(&self, content: text::Comment) -> Result<String> {
        self.repo
            .cache
            .comments()
            .can_edit(self.id, &self.user)
            .await?;

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
