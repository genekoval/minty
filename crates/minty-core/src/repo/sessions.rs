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
        let session = session.digest();

        self.repo.database.delete_user_session(&session).await?;
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
            .get(session.digest())
            .await?
            .ok_or(Error::Unauthenticated(None))
    }
}
