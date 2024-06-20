use super::{Cache, Cached, Id, Result, User};

use crate::SessionId;

use std::sync::Arc;

#[derive(Debug)]
pub struct Session {
    pub id: SessionId,
    user: Arc<Cached<User>>,
}

impl Session {
    pub fn new(id: SessionId, user: Arc<Cached<User>>) -> Self {
        Self { id, user }
    }

    pub fn user(&self) -> Arc<Cached<User>> {
        self.user.clone()
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
                if session.user.is_deleted() {
                    self.remove(id);
                    None
                } else {
                    Some(session)
                }
            }))
    }

    async fn on_miss(&self, id: SessionId) -> Result<Option<Session>> {
        if let (Some(user_id),) =
            self.cache.database.read_user_session(id.as_bytes()).await?
        {
            let Some(user) = self.cache.users().get(user_id).await? else {
                return Ok(None);
            };

            Ok(Some(Session::new(id, user)))
        } else {
            Ok(None)
        }
    }

    pub fn insert(
        &self,
        session: SessionId,
        user: Arc<Cached<User>>,
    ) -> Arc<Cached<Session>> {
        self.cache.sessions.insert(Session::new(session, user))
    }

    pub fn remove(&self, session: SessionId) {
        self.cache.sessions.remove(session);
    }
}
