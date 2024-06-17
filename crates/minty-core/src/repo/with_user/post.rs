mod edit;

pub use edit::Edit;

use crate::{
    cache::{self, User},
    Cached, Error, Repo, Result,
};

use minty::{text, CommentData, Uuid};
use std::sync::Arc;

pub struct Post<'a> {
    repo: &'a Repo,
    user: Arc<Cached<User>>,
    post: Arc<Cached<cache::Post>>,
}

impl<'a> Post<'a> {
    pub(super) fn new(
        repo: &'a Repo,
        user: Arc<Cached<User>>,
        post: Arc<Cached<cache::Post>>,
    ) -> Self {
        Self { repo, user, post }
    }

    pub fn edit(self) -> Result<Edit<'a>> {
        Edit::new(self.repo, self.user, self.post)
    }

    pub fn id(&self) -> Uuid {
        self.post.id
    }

    pub async fn add_comment(
        &self,
        content: text::Comment,
    ) -> Result<CommentData> {
        let comment = self
            .repo
            .database
            .create_comment(self.user.id, self.post.id, content.as_ref())
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "post_comment_post_id_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: self.post.id,
                        }),
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?;

        let result = CommentData {
            id: comment.id,
            user: self.user.preview(),
            content: comment.content.clone(),
            level: comment.level.try_into().unwrap(),
            created: comment.created,
        };

        self.post.add_comment(
            &self.post,
            &self.repo.cache,
            comment,
            self.user.clone(),
        );

        Ok(result)
    }
}
