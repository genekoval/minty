use crate::{Cached, Error, Repo, Result, Session, SessionId};

use std::sync::Arc;

pub struct Sessions<'a> {
    repo: &'a Repo,
}

impl<'a> Sessions<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn delete(&self, session: SessionId) -> Result<()> {
        self.repo
            .database
            .delete_user_session(session.as_bytes())
            .await?;

        self.repo.cache.sessions().remove(session);

        Ok(())
    }

    pub async fn get(
        &self,
        session: SessionId,
    ) -> Result<Arc<Cached<Session>>> {
        self.repo
            .cache
            .sessions()
            .get(session)
            .await?
            .ok_or(Error::Unauthenticated(None))
    }
}
