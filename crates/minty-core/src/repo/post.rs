use super::Repo;

use crate::{
    cache::{self, PostComments},
    db::PostObjects,
    error::Found,
    Cached, Error, Result,
};

use minty::{
    text::{self, Description, PostTitle},
    CommentData, DateTime, Modification, Uuid,
};
use std::sync::Arc;

pub struct Post<'a> {
    repo: &'a Repo,
    post: Arc<Cached<cache::Post>>,
}

impl<'a> Post<'a> {
    pub(super) fn new(repo: &'a Repo, post: Arc<Cached<cache::Post>>) -> Self {
        Self { repo, post }
    }

    pub fn id(&self) -> Uuid {
        self.post.id
    }

    async fn comments<F, R>(&self, f: F) -> Result<R>
    where
        F: Fn(PostComments<'_>) -> R,
    {
        self.post.comments(&self.post, &self.repo.cache, f).await
    }

    pub async fn add_comment(
        &self,
        user_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        let comment = self
            .repo
            .database
            .create_comment(user_id, self.post.id, content.as_ref())
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

        let user = self.repo.cache.users().get(user_id).await?;
        let result = CommentData {
            id: comment.id,
            user: user.as_ref().and_then(|user| user.preview()),
            content: comment.content.clone(),
            level: comment.level.try_into().unwrap(),
            created: comment.created,
        };

        self.post
            .add_comment(&self.post, &self.repo.cache, comment, user);

        Ok(result)
    }

    pub async fn add_objects(
        &self,
        objects: &[Uuid],
        destination: Option<Uuid>,
    ) -> Result<DateTime> {
        let mut tx = self.repo.database.begin().await?;

        let PostObjects { modified, objects } = tx
            .create_post_objects(self.post.id, objects, destination)
            .await?;

        self.repo
            .search
            .update_post_modified(self.post.id, modified)
            .await?;

        tx.commit().await?;

        let objects = self.repo.cache.objects().get_multiple(&objects).await?;
        self.post.add_objects(objects, modified);

        Ok(modified)
    }

    pub async fn add_tag(&self, tag: Uuid) -> Result<()> {
        let tag = self.repo.cache.tags().get(tag).await?.found("tag", tag)?;
        let mut tx = self.repo.database.begin().await?;

        tx.create_post_tag(self.post.id, tag.id)
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "post_tag_post_id_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: self.post.id,
                        }),
                        "post_tag_tag_id_fkey" => Some(Error::NotFound {
                            entity: "tag",
                            id: tag.id,
                        }),
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?;

        self.repo.search.add_post_tag(self.post.id, tag.id).await?;

        tx.commit().await?;

        self.post.add_tag(tag);

        Ok(())
    }

    pub async fn add_related_post(&self, related: Uuid) -> Result<()> {
        if self.post.id == related {
            return Err(Error::InvalidInput(
                "post cannot be related to itself".into(),
            ));
        }

        let (posts,) = self
            .repo
            .database
            .create_related_post(self.post.id, related)
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "related_post_post_id_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: self.post.id,
                        }),
                        "related_post_related_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: related,
                        }),
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?;

        self.post.set_related_posts(posts);

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        let mut tx = self.repo.database.begin().await?;

        tx.delete_post(self.post.id)
            .await?
            .found("post", self.post.id)?;

        self.repo.search.delete_post(self.post.id).await?;

        tx.commit().await?;

        self.repo.cache.posts().remove(&self.post);

        Ok(())
    }

    pub async fn delete_objects(&self, objects: &[Uuid]) -> Result<DateTime> {
        let mut tx = self.repo.database.begin().await?;

        let modified = tx.delete_post_objects(self.post.id, objects).await?.0;
        self.repo
            .search
            .update_post_modified(self.post.id, modified)
            .await?;

        tx.commit().await?;

        self.post.delete_objects(objects, modified);

        Ok(modified)
    }

    pub async fn delete_tag(&self, tag_id: Uuid) -> Result<bool> {
        let mut tx = self.repo.database.begin().await?;

        let found = tx.delete_post_tag(self.post.id, tag_id).await?;
        if found {
            self.repo
                .search
                .remove_post_tag(self.post.id, tag_id)
                .await?;
        }

        tx.commit().await?;

        self.post.delete_tag(tag_id);

        Ok(found)
    }

    pub async fn delete_related_post(&self, related: Uuid) -> Result<()> {
        let posts = self
            .repo
            .database
            .delete_related_post(self.post.id, related)
            .await?
            .0
            .found("post", related)?;

        self.post.set_related_posts(posts);

        Ok(())
    }

    pub async fn get(&self) -> Result<minty::Post> {
        self.post
            .model(&self.repo.cache)
            .await?
            .found("post", self.post.id)
    }

    pub async fn get_comments(&self) -> Result<Vec<CommentData>> {
        self.comments(|comments| comments.get_all()).await
    }

    pub async fn publish(&self) -> Result<()> {
        let mut tx = self.repo.database.begin().await?;

        let timestamp = tx.publish_post(self.post.id).await?.0;
        self.repo
            .search
            .publish_post(self.post.id, timestamp)
            .await?;

        tx.commit().await?;

        self.post.publish(timestamp);

        Ok(())
    }

    pub async fn set_description(
        &self,
        description: Description,
    ) -> Result<Modification<String>> {
        let description: String = description.into();
        let mut tx = self.repo.database.begin().await?;

        let (modified,) = tx
            .update_post_description(self.post.id, &description)
            .await?
            .found("post", self.post.id)?;

        self.repo
            .search
            .update_post_description(self.post.id, &description, modified)
            .await?;

        tx.commit().await?;

        self.post.set_description(description.clone(), modified);

        Ok(Modification {
            date_modified: modified,
            new_value: description,
        })
    }

    pub async fn set_title(
        &self,
        title: PostTitle,
    ) -> Result<Modification<String>> {
        let title: String = title.into();
        let mut tx = self.repo.database.begin().await?;

        let (modified,) = tx
            .update_post_title(self.post.id, &title)
            .await?
            .found("post", self.post.id)?;

        self.repo
            .search
            .update_post_title(self.post.id, &title, modified)
            .await?;

        tx.commit().await?;

        self.post.set_title(title.clone(), modified);

        Ok(Modification {
            date_modified: modified,
            new_value: title,
        })
    }
}
