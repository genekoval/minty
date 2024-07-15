use super::{Cached, User};

use crate::db;

use minty::{CommentData, DateTime, Uuid};
use std::{mem, sync::Arc};

#[derive(Debug)]
pub struct Comment {
    pub id: Uuid,
    pub user: Option<Arc<Cached<User>>>,
    pub level: u8,
    pub content: String,
    pub created: DateTime,
    pub children: Vec<Self>,
}

impl Comment {
    pub fn new(comment: db::Comment, user: Option<Arc<Cached<User>>>) -> Self {
        Self {
            id: comment.id,
            user,
            level: comment.level.try_into().unwrap(),
            content: comment.content,
            created: comment.created,
            children: vec![],
        }
    }

    pub fn data(&self) -> CommentData {
        CommentData {
            id: self.id,
            user: self.user.as_ref().and_then(|user| user.preview()),
            content: self.content.clone(),
            level: self.level,
            created: self.created,
        }
    }

    pub fn model(
        &self,
        post_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Option<minty::Comment> {
        if self.content.is_empty() && self.children.is_empty() {
            None
        } else {
            Some(minty::Comment {
                id: self.id,
                user: self.user.as_ref().and_then(|user| user.preview()),
                post_id,
                parent_id,
                level: self.level,
                content: self.content.clone(),
                created: self.created,
            })
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty() && self.children.is_empty()
    }

    pub fn count(&self) -> usize {
        self.children
            .iter()
            .map(|child| child.count())
            .sum::<usize>()
            + 1 // Add one for self
    }

    fn decrement_count(&self, recursive: bool) {
        if self.content.is_empty() {
            return;
        }

        if let Some(user) = &self.user {
            user.update(|user| user.comment_count -= 1);
        }

        if recursive {
            for child in &self.children {
                child.decrement_count(true);
            }
        }
    }

    pub fn delete(&mut self, recursive: bool) -> Vec<Self> {
        self.decrement_count(recursive);
        self.content = String::new();

        if recursive {
            mem::take(&mut self.children)
        } else {
            Vec::new()
        }
    }

    pub fn push(&mut self, comment: Self, path: &mut Vec<usize>) {
        if comment.level == self.level + 1 {
            path.push(self.children.len());
            self.children.push(comment);
        } else {
            path.push(self.children.len() - 1);
            self.children.last_mut().unwrap().push(comment, path);
        }
    }

    pub fn reply(&mut self, reply: Self) -> usize {
        self.children.push(reply);
        self.children.len() - 1
    }

    pub fn thread(&self, result: &mut Vec<CommentData>) {
        result.push(self.data());

        self.children
            .iter()
            .rev()
            .filter(|child| !child.is_empty())
            .for_each(|child| child.thread(result));
    }
}
