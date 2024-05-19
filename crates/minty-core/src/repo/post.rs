use super::Repo;

use crate::{comment, error::Found, Error, Result};

use minty::{
    text::{self, Description, PostTitle},
    CommentData, DateTime, Modification, Uuid,
};

pub struct Post<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Post<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn add_comment(
        &self,
        user_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        Ok(self
            .repo
            .database
            .create_comment(user_id, self.id, content.as_ref())
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "post_comment_post_id_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: self.id,
                        }),
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?
            .into())
    }

    pub async fn add_objects(
        &self,
        objects: &[Uuid],
        destination: Option<Uuid>,
    ) -> Result<DateTime> {
        let mut tx = self.repo.database.begin().await?;

        let modified = tx
            .create_post_objects(self.id, objects, destination)
            .await?
            .0;
        self.repo
            .search
            .update_post_modified(self.id, modified)
            .await?;

        tx.commit().await?;
        Ok(modified)
    }

    pub async fn add_tag(&self, tag_id: Uuid) -> Result<()> {
        let mut tx = self.repo.database.begin().await?;

        tx.create_post_tag(self.id, tag_id).await.map_err(|err| {
            err.as_database_error()
                .and_then(|e| e.constraint())
                .and_then(|constraint| match constraint {
                    "post_tag_post_id_fkey" => Some(Error::NotFound {
                        entity: "post",
                        id: self.id,
                    }),
                    "post_tag_tag_id_fkey" => Some(Error::NotFound {
                        entity: "tag",
                        id: tag_id,
                    }),
                    _ => None,
                })
                .unwrap_or_else(|| err.into())
        })?;
        self.repo.search.add_post_tag(self.id, tag_id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn add_related_post(&self, related: Uuid) -> Result<()> {
        if self.id == related {
            return Err(Error::InvalidInput(
                "post cannot be related to itself".into(),
            ));
        }

        self.repo
            .database
            .create_related_post(self.id, related)
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "related_post_post_id_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: self.id,
                        }),
                        "related_post_related_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: related,
                        }),
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?;

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        let mut tx = self.repo.database.begin().await?;

        tx.delete_post(self.id).await?.found("post", self.id)?;
        self.repo.search.delete_post(self.id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_objects(&self, objects: &[Uuid]) -> Result<DateTime> {
        let mut tx = self.repo.database.begin().await?;

        let modified = tx.delete_post_objects(self.id, objects).await?.0;
        self.repo
            .search
            .update_post_modified(self.id, modified)
            .await?;

        tx.commit().await?;
        Ok(modified)
    }

    pub async fn delete_tag(&self, tag_id: Uuid) -> Result<bool> {
        let mut tx = self.repo.database.begin().await?;

        let found = tx.delete_post_tag(self.id, tag_id).await?;
        if found {
            self.repo.search.remove_post_tag(self.id, tag_id).await?;
        }

        tx.commit().await?;
        Ok(found)
    }

    pub async fn delete_related_post(&self, related: Uuid) -> Result<bool> {
        Ok(self
            .repo
            .database
            .delete_related_post(self.id, related)
            .await?)
    }

    pub async fn get(&self) -> Result<minty::Post> {
        let post = self
            .repo
            .database
            .read_post(self.id)
            .await?
            .found("post", self.id)?;

        Ok(minty::Post {
            id: post.id,
            poster: post.poster.map(Into::into),
            title: post.title,
            description: post.description,
            visibility: post.visibility.into(),
            created: post.created,
            modified: post.modified,
            objects: self.repo.bucket.get_object_previews(post.objects).await?,
            posts: self.repo.posts().build(post.posts).await?,
            tags: post.tags.into_iter().map(|tag| tag.into()).collect(),
            comment_count: post.comment_count,
        })
    }

    pub async fn get_comments(&self) -> Result<Vec<CommentData>> {
        let comments = self.repo.database.read_comments(self.id).await?;
        Ok(comment::build_tree(comments))
    }

    pub async fn publish(&self) -> Result<()> {
        let mut tx = self.repo.database.begin().await?;

        let timestamp = tx.publish_post(self.id).await?.0;
        self.repo.search.publish_post(self.id, timestamp).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn set_description(
        &self,
        description: Description,
    ) -> Result<Modification<String>> {
        let mut tx = self.repo.database.begin().await?;

        let (modified,) = tx
            .update_post_description(self.id, description.as_ref())
            .await?
            .found("post", self.id)?;

        self.repo
            .search
            .update_post_description(self.id, description.as_ref(), modified)
            .await?;

        tx.commit().await?;
        Ok(Modification {
            date_modified: modified,
            new_value: description.into(),
        })
    }

    pub async fn set_title(
        &self,
        title: PostTitle,
    ) -> Result<Modification<String>> {
        let mut tx = self.repo.database.begin().await?;

        let (modified,) = tx
            .update_post_title(self.id, title.as_ref())
            .await?
            .found("post", self.id)?;

        self.repo
            .search
            .update_post_title(self.id, title.as_ref(), modified)
            .await?;

        tx.commit().await?;
        Ok(Modification {
            date_modified: modified,
            new_value: title.into(),
        })
    }
}
