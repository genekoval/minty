use super::Cache;

use crate::SessionId;

use minty::Uuid;

#[derive(Debug)]
pub struct Sessions<'a> {
    cache: &'a Cache,
}

impl<'a> Sessions<'a> {
    pub(super) fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }

    pub async fn get_user(&self, session: SessionId) -> Option<Uuid> {
        self.cache.sessions.get(&session).await
    }

    pub async fn insert(&self, session: SessionId, user: Uuid) {
        self.cache.sessions.insert(session, user).await;
    }

    pub async fn remove(&self, session: SessionId) {
        self.cache.sessions.invalidate(&session).await;
    }
}
