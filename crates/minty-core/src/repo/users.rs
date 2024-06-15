use super::{Repo, User};

use crate::{
    cache::{Cached, Session},
    db::Password,
    error::Found,
    Error, Result, SessionId,
};

use minty::{Login, ProfileQuery, SearchResult, SignUp, UserPreview, Uuid};
use std::sync::Arc;

pub struct Users<'a> {
    repo: &'a Repo,
}

impl<'a> Users<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn add(&self, info: SignUp) -> Result<User> {
        let name = info.username.as_ref();
        let email = info.email.as_ref();
        let password = self.repo.auth.hash_password(info.password)?;

        let mut tx = self.repo.database.begin().await?;

        let user =
            tx.create_user(name, email, &password)
                .await
                .map_err(|err| {
                    err.as_database_error()
                        .and_then(|e| e.constraint())
                        .and_then(|constraint| match constraint {
                            "user_account_email_key" => {
                                Some(Error::AlreadyExists {
                                    entity: "user",
                                    identifier: format!(
                                        "email address '{email}'"
                                    ),
                                })
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| err.into())
                })?;

        self.repo.search.add_user_alias(user.id, name).await?;

        tx.commit().await?;

        let user = self.repo.cache.users().insert(user);
        Ok(User::new(self.repo, user))
    }

    pub async fn authenticate(&self, login: &Login) -> Result<SessionId> {
        const ERROR: Option<&str> = Some("invalid credentials");

        let Some(Password { user_id, password }) =
            self.repo.database.read_user_password(&login.email).await?
        else {
            return Err(Error::Unauthenticated(ERROR));
        };

        let authenticated =
            self.repo.auth.verify_password(&login.password, &password)?;

        if authenticated {
            let user = self
                .repo
                .cache
                .users()
                .get(user_id)
                .await?
                .found("user", user_id)?;
            self.repo.user(user).create_session().await
        } else {
            Err(Error::Unauthenticated(ERROR))
        }
    }

    pub async fn delete_session(&self, session: SessionId) -> Result<()> {
        self.repo
            .database
            .delete_user_session(session.as_bytes())
            .await?;

        self.repo.cache.sessions().remove(session);

        Ok(())
    }

    pub async fn find(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<UserPreview>> {
        let results = self
            .repo
            .search
            .find_entities(&self.repo.search.indices.user, query)
            .await?;

        let users = self
            .repo
            .cache
            .users()
            .get_multiple(&results.hits)
            .await?
            .into_iter()
            .filter_map(|user| user.preview())
            .collect();

        Ok(SearchResult {
            total: results.total,
            hits: users,
        })
    }

    pub async fn get(&self, id: Uuid) -> Result<User> {
        let user = self.repo.cache.users().get(id).await?.found("user", id)?;
        Ok(User::new(self.repo, user))
    }

    pub async fn get_session(
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
