use super::{Cache, Cached, Id, Result, User};

use crate::auth::Digest;

use chrono::Local;
use minty::DateTime;
use std::sync::Arc;

#[derive(Debug)]
pub struct Session {
    pub id: Digest,
    user: Arc<Cached<User>>,
    expiration: DateTime,
}

impl Session {
    pub fn new(
        id: Digest,
        user: Arc<Cached<User>>,
        expiration: DateTime,
    ) -> Self {
        Self {
            id,
            user,
            expiration,
        }
    }

    pub fn user(&self) -> Arc<Cached<User>> {
        self.user.clone()
    }

    fn is_expired(&self) -> bool {
        Local::now() >= self.expiration
    }

    fn is_valid(&self) -> bool {
        !self.user.is_deleted() && !self.is_expired()
    }
}

impl Id for Session {
    type Id = Digest;

    fn id(&self) -> Self::Id {
        self.id
    }
}

pub struct Sessions<'a> {
    cache: &'a Cache,
}

impl<'a> Sessions<'a> {
    pub(super) fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }

    pub async fn get(
        &self,
        id: Digest,
    ) -> Result<Option<Arc<Cached<Session>>>> {
        Ok(self
            .cache
            .sessions
            .get(id, || async { self.on_miss(id).await })
            .await?
            .and_then(|session| {
                if session.is_valid() {
                    Some(session)
                } else {
                    self.remove(id);
                    None
                }
            }))
    }

    async fn on_miss(&self, id: Digest) -> Result<Option<Session>> {
        let Some(session) = self.cache.database.read_user_session(&id).await?
        else {
            return Ok(None);
        };

        let Some(user) = self.cache.users().get(session.user_id).await? else {
            return Ok(None);
        };

        Ok(Some(Session::new(id, user, session.expiration)))
    }

    pub fn insert(
        &self,
        session: Digest,
        user: Arc<Cached<User>>,
        expiration: DateTime,
    ) -> Arc<Cached<Session>> {
        self.cache
            .sessions
            .insert(Session::new(session, user, expiration))
    }

    pub fn remove(&self, session: Digest) {
        self.cache.sessions.remove(session);
    }
}
