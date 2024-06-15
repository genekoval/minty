use super::Repo;

use crate::{cache, error::Found, Cached, Result, SessionId};

use minty::{
    text::{Description, Email, Name, Password},
    ProfileName, Source, Url,
};
use std::sync::Arc;

pub struct User<'a> {
    repo: &'a Repo,
    user: Arc<Cached<cache::User>>,
}

impl<'a> User<'a> {
    pub(super) fn new(repo: &'a Repo, user: Arc<Cached<cache::User>>) -> Self {
        Self { repo, user }
    }

    pub async fn add_alias(&self, alias: Name) -> Result<ProfileName> {
        let names = self
            .repo
            .entity(self.user.id)
            .add_alias(alias, "user", &self.repo.search.indices.user)
            .await?;

        self.user.update(|user| user.profile.set_names(&names));

        Ok(names)
    }

    pub async fn add_source(&self, url: &Url) -> Result<Source> {
        let source =
            self.repo.entity(self.user.id).add_link("user", url).await?;

        self.user
            .update(|user| user.profile.add_source(source.clone()));

        Ok(source)
    }

    pub async fn create_session(&self) -> Result<SessionId> {
        let session = SessionId::new();

        self.repo
            .database
            .create_user_session(self.user.id, session.as_bytes())
            .await?;

        self.repo
            .cache
            .sessions()
            .insert(session, self.user.clone());

        Ok(session)
    }

    pub async fn delete(&self) -> Result<()> {
        self.repo
            .entity(self.user.id)
            .delete("user", &self.repo.search.indices.user)
            .await?;

        self.repo.cache.users().remove(&self.user);

        Ok(())
    }

    pub async fn delete_alias(&self, alias: &str) -> Result<ProfileName> {
        let names = self
            .repo
            .entity(self.user.id)
            .delete_alias(alias, "user", &self.repo.search.indices.user)
            .await?;

        self.user.update(|user| user.profile.set_names(&names));

        Ok(names)
    }

    pub async fn delete_source(&self, source_id: i64) -> Result<bool> {
        let deleted = self
            .repo
            .database
            .delete_entity_link(self.user.id, source_id)
            .await?;

        if deleted {
            self.user
                .update(|user| user.profile.delete_source(source_id));
        }

        Ok(deleted)
    }

    pub async fn delete_sources<S>(&self, sources: &[S]) -> Result<()>
    where
        S: AsRef<str>,
    {
        let ids = self
            .repo
            .entity(self.user.id)
            .delete_sources(sources)
            .await?;

        self.user.update(|user| user.profile.delete_sources(&ids));

        Ok(())
    }

    pub fn get(&self) -> Result<minty::User> {
        self.user.model().found("user", self.user.id)
    }

    pub async fn set_description(
        &self,
        description: Description,
    ) -> Result<String> {
        let description: String = description.into();

        self.repo
            .database
            .update_entity_description(self.user.id, &description)
            .await?
            .found("user", self.user.id)?;

        self.user
            .update(|user| user.profile.description.clone_from(&description));

        Ok(description)
    }

    pub async fn set_admin(&self, admin: bool) -> Result<()> {
        self.repo
            .database
            .update_admin(self.user.id, admin)
            .await?
            .found("user", self.user.id)?;

        self.user.update(|user| user.admin = admin);

        Ok(())
    }

    pub async fn set_email(&self, email: Email) -> Result<()> {
        let email: String = email.into();

        self.repo
            .database
            .update_user_email(self.user.id, &email)
            .await?
            .found("user", self.user.id)?;

        self.user.update(|user| user.email = email);

        Ok(())
    }

    pub async fn set_password(&self, password: Password) -> Result<()> {
        let password = self.repo.auth.hash_password(password)?;

        self.repo
            .database
            .update_user_password(self.user.id, &password)
            .await?
            .found("user", self.user.id)?;

        Ok(())
    }

    pub async fn set_name(&self, new_name: Name) -> Result<ProfileName> {
        let names = self
            .repo
            .entity(self.user.id)
            .set_name(new_name, "user", &self.repo.search.indices.user)
            .await?;

        self.user.update(|user| user.profile.set_names(&names));

        Ok(names)
    }
}
