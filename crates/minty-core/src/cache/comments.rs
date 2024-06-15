use super::{Cache, Cached, Comment, CommentEntry, User};

use crate::{db, error::Found, Result};

use minty::{CommentData, Uuid};
use std::sync::Arc;

pub struct Comments<'a> {
    cache: &'a Cache,
}

impl<'a> Comments<'a> {
    pub(super) fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }

    fn comment<F, R>(&self, id: Uuid, f: F) -> Option<R>
    where
        F: FnOnce(&mut Comment) -> R,
    {
        let CommentEntry(post, path) = self.cache.comments.get(id)?;
        post.comment(&path, f)
    }

    pub fn delete(&self, id: Uuid, recursive: bool) {
        if let Some(CommentEntry(post, path)) = self.cache.comments.get(id) {
            let comments = post.delete_comment(&path, recursive);

            if !comments.is_empty() {
                self.cache.comments.delete(comments);
            }
        }
    }

    pub async fn get(&self, id: Uuid) -> Result<minty::Comment> {
        if let Some(CommentEntry(post, path)) = self.cache.comments.get(id) {
            return post
                .comments(&post, self.cache, |comments| comments.get(&path))
                .await?
                .found("comment", id);
        }

        let post = self
            .cache
            .database
            .read_comment_post(id)
            .await?
            .0
            .found("comment", id)?;

        let post = self.cache.posts().get(post).await?.found("comment", id)?;

        post.comments(&post, self.cache, |comments| {
            let path = self.cache.comments.path(id);
            comments.get(&path)
        })
        .await?
        .found("comment", id)
    }

    pub fn reply(
        &self,
        parent: Uuid,
        comment: db::Comment,
        user: Arc<Cached<User>>,
    ) -> CommentData {
        let comment = Comment::new(comment, Some(user));
        let data = comment.data();

        if let Some(CommentEntry(post, path)) = self.cache.comments.get(parent)
        {
            if let Some(index) = post.reply(&path, comment) {
                let mut path = path.into_vec();
                path.push(index);
                let path = path.into_boxed_slice();

                self.cache.comments.insert(data.id, &post, path);
            }
        }

        data
    }

    pub fn update(&self, id: Uuid, content: &String) {
        self.comment(id, |comment| comment.content.clone_from(content));
    }
}
