use super::{Cache, CacheLock, Cached, Id, Result};

use crate::{db, Error};

use minty::{EntityProfile, UserPreview, Uuid};
use std::sync::Arc;

#[derive(Debug)]
pub struct UserMut {
    pub email: String,
    pub admin: bool,
    pub profile: EntityProfile,
    pub post_count: u32,
    pub comment_count: u32,
    pub tag_count: u32,
}

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    mutable: CacheLock<UserMut>,
}

impl User {
    fn is_admin(&self) -> bool {
        self.mutable.map(|user| user.admin).unwrap_or(false)
    }

    pub fn model(&self) -> Option<minty::User> {
        self.mutable.map(|user| minty::User {
            id: self.id,
            email: user.email.clone(),
            admin: user.admin,
            profile: user.profile.clone(),
            post_count: user.post_count,
            comment_count: user.comment_count,
            tag_count: user.tag_count,
        })
    }

    pub fn preview(&self) -> Option<UserPreview> {
        self.mutable.map(|user| UserPreview {
            id: self.id,
            name: user.profile.name.clone(),
            avatar: user.profile.avatar,
        })
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut UserMut),
    {
        self.mutable.update(f);
    }

    pub fn delete(&self) {
        self.mutable.delete();
    }

    pub fn deny_permission(&self) -> Result<()> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    }
}

impl From<db::User> for User {
    fn from(value: db::User) -> Self {
        Self {
            id: value.id,
            mutable: CacheLock::new(UserMut {
                email: value.email,
                admin: value.admin,
                profile: value.profile.into(),
                post_count: value.post_count,
                comment_count: value.comment_count,
                tag_count: value.tag_count,
            }),
        }
    }
}

impl Id for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

pub struct Users<'a> {
    cache: &'a Cache,
}

impl<'a> Users<'a> {
    pub(super) fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Arc<Cached<User>>>> {
        self.cache
            .users
            .get(id, || async { self.on_miss(id).await })
            .await
    }

    pub async fn get_multiple(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Arc<Cached<User>>>> {
        self.cache
            .users
            .get_multiple(ids, |ids| async move {
                Ok(self
                    .cache
                    .database
                    .read_users(&ids)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect())
            })
            .await
    }

    async fn on_miss(&self, id: Uuid) -> Result<Option<User>> {
        Ok(self.cache.database.read_user(id).await?.map(Into::into))
    }

    pub fn insert(&self, user: db::User) -> Arc<Cached<User>> {
        self.cache.users.insert(user.into())
    }

    pub fn remove(&self, user: &Arc<Cached<User>>) {
        user.delete();
        self.cache.users.remove(user.id);
    }
}
