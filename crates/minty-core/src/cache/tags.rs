use super::{Cache, CacheLock, Cached, Id, Result, User};

use crate::db;

use minty::{EntityProfile, TagPreview, Uuid};
use std::sync::Arc;

#[derive(Debug)]
pub struct TagMut {
    pub profile: EntityProfile,
    pub post_count: u32,
}

#[derive(Debug)]
pub struct Tag {
    pub id: Uuid,
    pub creator: Option<Arc<Cached<User>>>,
    mutable: CacheLock<TagMut>,
}

impl Tag {
    async fn new(tag: db::Tag, cache: &Cache) -> Self {
        let creator = if let Some(creator) = tag.creator {
            cache.users().get(creator).await.ok().flatten()
        } else {
            None
        };

        Self::with_creator(tag, creator)
    }

    fn with_creator(tag: db::Tag, creator: Option<Arc<Cached<User>>>) -> Self {
        Self {
            id: tag.id,
            creator,
            mutable: CacheLock::new(TagMut {
                profile: tag.profile.into(),
                post_count: tag.post_count,
            }),
        }
    }

    pub fn can_edit(&self, user: &Arc<Cached<User>>) -> Result<()> {
        let creator = self.creator.as_ref().map(|user| user.id);

        if creator == Some(user.id) {
            Ok(())
        } else {
            user.deny_permission()
        }
    }

    pub fn model(&self) -> Option<minty::Tag> {
        self.mutable.map(|tag| minty::Tag {
            id: self.id,
            profile: tag.profile.clone(),
            creator: self.creator.as_ref().and_then(|user| user.preview()),
            post_count: tag.post_count,
        })
    }

    pub fn preview(&self) -> Option<TagPreview> {
        self.mutable.map(|tag| TagPreview {
            id: self.id,
            name: tag.profile.name.clone(),
            avatar: tag.profile.avatar,
        })
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut TagMut),
    {
        self.mutable.update(f);
    }

    pub fn delete(&self) {
        if let Some(creator) = &self.creator {
            creator.update(|user| user.tag_count -= 1);
        }

        self.mutable.delete();
    }

    pub fn is_deleted(&self) -> bool {
        self.mutable.is_deleted()
    }
}

impl Id for Tag {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

pub struct Tags<'a> {
    cache: &'a Cache,
}

impl<'a> Tags<'a> {
    pub(super) fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Arc<Cached<Tag>>>> {
        self.cache
            .tags
            .get(id, || async {
                if let Some(tag) = self.cache.database.read_tag(id).await? {
                    Ok(Some(Tag::new(tag, self.cache).await))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    pub async fn get_multiple(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Arc<Cached<Tag>>>> {
        self.cache
            .tags
            .get_multiple(ids, |ids| async move {
                let tags = self.cache.database.read_tags(&ids).await?;
                let mut result = Vec::with_capacity(tags.len());

                for tag in tags {
                    result.push(Tag::new(tag, self.cache).await);
                }

                Ok(result)
            })
            .await
    }

    pub async fn previews(&self, ids: &[Uuid]) -> Result<Vec<TagPreview>> {
        Ok(self
            .get_multiple(ids)
            .await?
            .into_iter()
            .filter_map(|tag| tag.preview())
            .collect())
    }

    pub fn insert(
        &self,
        tag: db::Tag,
        creator: Arc<Cached<User>>,
    ) -> Arc<Cached<Tag>> {
        creator.update(|user| user.tag_count += 1);

        self.cache
            .tags
            .insert(Tag::with_creator(tag, Some(creator)))
    }

    pub fn remove(&self, tag: &Arc<Cached<Tag>>) {
        tag.delete();
        self.cache.tags.remove(tag.id);
    }
}
