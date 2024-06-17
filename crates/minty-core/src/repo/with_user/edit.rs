use crate::{cache::User, error::Found, Cached, Repo, Result};

use minty::{
    text::{Description, Email, Name, Password},
    ProfileName, Source, Url,
};
use std::sync::Arc;

pub struct Edit<'a> {
    repo: &'a Repo,
    user: Arc<Cached<User>>,
}

impl<'a> Edit<'a> {
    pub(super) fn new(repo: &'a Repo, user: Arc<Cached<User>>) -> Self {
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
