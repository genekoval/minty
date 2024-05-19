use super::Repo;

use crate::{error::Found, Result, SessionId};

use minty::{
    text::{Description, Email, Name, Password},
    ProfileName, Source, Url, Uuid,
};

pub struct User<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> User<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn add_alias(&self, alias: Name) -> Result<ProfileName> {
        self.repo
            .entity(self.id)
            .add_alias(alias, "user", &self.repo.search.indices.user)
            .await
    }

    pub async fn add_source(&self, url: &Url) -> Result<Source> {
        self.repo.entity(self.id).add_link("user", url).await
    }

    pub async fn create_session(&self) -> Result<SessionId> {
        let session = SessionId::new();
        self.repo
            .database
            .create_user_session(self.id, session.as_bytes())
            .await?;
        Ok(session)
    }

    pub async fn delete(&self) -> Result<()> {
        self.repo
            .entity(self.id)
            .delete("user", &self.repo.search.indices.user)
            .await
    }

    pub async fn delete_alias(&self, alias: &str) -> Result<ProfileName> {
        self.repo
            .entity(self.id)
            .delete_alias(alias, "user", &self.repo.search.indices.user)
            .await
    }

    pub async fn delete_source(&self, source_id: i64) -> Result<bool> {
        Ok(self
            .repo
            .database
            .delete_entity_link(self.id, source_id)
            .await?)
    }

    pub async fn delete_sources<S>(&self, sources: &[S]) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.repo.entity(self.id).delete_sources(sources).await
    }

    pub async fn get(&self) -> Result<minty::User> {
        Ok(self
            .repo
            .database
            .read_user(self.id)
            .await?
            .found("user", self.id)?
            .into())
    }

    pub async fn set_description(
        &self,
        description: Description,
    ) -> Result<String> {
        self.repo
            .database
            .update_entity_description(self.id, description.as_ref())
            .await?
            .found("user", self.id)?;

        Ok(description.into())
    }

    pub async fn set_email(&self, email: Email) -> Result<()> {
        self.repo
            .database
            .update_user_email(self.id, email.as_ref())
            .await?
            .found("user", self.id)?;

        Ok(())
    }

    pub async fn set_password(&self, password: Password) -> Result<()> {
        let password = self.repo.auth.hash_password(password)?;

        self.repo
            .database
            .update_user_password(self.id, &password)
            .await?
            .found("user", self.id)?;

        Ok(())
    }

    pub async fn set_name(&self, new_name: Name) -> Result<ProfileName> {
        self.repo
            .entity(self.id)
            .set_name(new_name, "user", &self.repo.search.indices.user)
            .await
    }
}
