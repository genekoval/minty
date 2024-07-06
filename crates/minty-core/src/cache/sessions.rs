use super::{Cache, Cached, Id, Result, User};

use crate::SessionId;

use chrono::Local;
use minty::DateTime;
use std::sync::Arc;

#[derive(Debug)]
pub struct Session {
    pub id: SessionId,
    user: Arc<Cached<User>>,
    expiration: DateTime,
}

impl Session {
    pub fn new(
        id: SessionId,
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
    type Id = SessionId;

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
        id: SessionId,
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

    async fn on_miss(&self, id: SessionId) -> Result<Option<Session>> {
        let Some(session) =
            self.cache.database.read_user_session(id.as_bytes()).await?
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
        session: SessionId,
        user: Arc<Cached<User>>,
        expiration: DateTime,
    ) -> Arc<Cached<Session>> {
        self.cache
            .sessions
            .insert(Session::new(session, user, expiration))
    }

    pub fn remove(&self, session: SessionId) {
        self.cache.sessions.remove(session);
    }
}
