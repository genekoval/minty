use super::Repo;

use crate::{db::Password, Error, Result, SessionId};

use log::debug;
use minty::{Login, ProfileQuery, SearchResult, SignUp, UserPreview, Uuid};

pub struct Users<'a> {
    repo: &'a Repo,
}

impl<'a> Users<'a> {
    pub(super) fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn add(&self, info: SignUp) -> Result<Uuid> {
        let name = info.username.as_ref();
        let email = info.email.as_ref();
        let password = self.repo.auth.hash_password(info.password)?;

        let mut tx = self.repo.database.begin().await?;

        let id = tx
            .create_user(name, email, &password)
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "user_account_email_key" => {
                            Some(Error::AlreadyExists {
                                entity: "user",
                                identifier: format!("email address '{email}'"),
                            })
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?
            .0;

        self.repo.search.add_user_alias(id, name).await?;

        tx.commit().await?;
        Ok(id)
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
            self.repo.user(user_id).create_session().await
        } else {
            Err(Error::Unauthenticated(ERROR))
        }
    }

    pub async fn delete_session(&self, session: SessionId) -> Result<()> {
        self.repo
            .database
            .delete_user_session(session.as_bytes())
            .await?;

        self.repo.sessions.remove(session);

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
            .database
            .read_user_previews(&results.hits)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(SearchResult {
            total: results.total,
            hits: users,
        })
    }

    pub async fn get_session(&self, session: SessionId) -> Result<Uuid> {
        if let Some(user_id) = self.repo.sessions.get(session) {
            debug!("session cache hit for user {user_id}");
            Ok(user_id)
        } else if let (Some(user_id),) = self
            .repo
            .database
            .read_user_session(session.as_bytes())
            .await?
        {
            self.repo.sessions.insert(session, user_id);
            Ok(user_id)
        } else {
            Err(Error::Unauthenticated(None))
        }
    }
}
