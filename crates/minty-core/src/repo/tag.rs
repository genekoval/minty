use super::Repo;

use crate::{error::Found, Result};

use minty::{
    text::{Description, Name},
    ProfileName, Source, Url, Uuid,
};

pub struct Tag<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Tag<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn add_alias(&self, alias: Name) -> Result<ProfileName> {
        self.repo
            .entity(self.id)
            .add_alias(alias, "tag", &self.repo.search.indices.tag)
            .await
    }

    pub async fn add_source(&self, url: &Url) -> Result<Source> {
        self.repo.entity(self.id).add_link("tag", url).await
    }

    pub async fn delete(&self) -> Result<()> {
        self.repo
            .entity(self.id)
            .delete("tag", &self.repo.search.indices.tag)
            .await
    }

    pub async fn delete_alias(&self, alias: &str) -> Result<ProfileName> {
        self.repo
            .entity(self.id)
            .delete_alias(alias, "tag", &self.repo.search.indices.tag)
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

    pub async fn get(&self) -> Result<minty::Tag> {
        Ok(self
            .repo
            .database
            .read_tag(self.id)
            .await?
            .found("tag", self.id)?
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
            .found("tag", self.id)?;

        Ok(description.into())
    }

    pub async fn set_name(&self, new_name: Name) -> Result<ProfileName> {
        self.repo
            .entity(self.id)
            .set_name(new_name, "tag", &self.repo.search.indices.tag)
            .await
    }
}
