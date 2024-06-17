use crate::{cache, error::Found, Cached, Repo, Result};

use minty::{
    text::{Description, Name},
    ProfileName, Source, Url, Uuid,
};
use std::sync::Arc;

pub struct Tag<'a> {
    repo: &'a Repo,
    tag: Arc<Cached<cache::Tag>>,
}

impl<'a> Tag<'a> {
    pub(super) fn new(repo: &'a Repo, tag: Arc<Cached<cache::Tag>>) -> Self {
        Self { repo, tag }
    }

    pub fn id(&self) -> Uuid {
        self.tag.id
    }

    pub async fn add_alias(&self, alias: Name) -> Result<ProfileName> {
        let names = self
            .repo
            .entity(self.tag.id)
            .add_alias(alias, "tag", &self.repo.search.indices.tag)
            .await?;

        self.tag.update(|tag| tag.profile.set_names(&names));

        Ok(names)
    }

    pub async fn add_source(&self, url: &Url) -> Result<Source> {
        let source = self.repo.entity(self.tag.id).add_link("tag", url).await?;

        self.tag
            .update(|tag| tag.profile.add_source(source.clone()));

        Ok(source)
    }

    pub async fn delete(&self) -> Result<()> {
        self.repo
            .entity(self.tag.id)
            .delete("tag", &self.repo.search.indices.tag)
            .await?;

        self.repo.cache.tags().remove(&self.tag);

        Ok(())
    }

    pub async fn delete_alias(&self, alias: &str) -> Result<ProfileName> {
        let names = self
            .repo
            .entity(self.tag.id)
            .delete_alias(alias, "tag", &self.repo.search.indices.tag)
            .await?;

        self.tag.update(|tag| tag.profile.set_names(&names));

        Ok(names)
    }

    pub async fn delete_source(&self, source_id: i64) -> Result<bool> {
        let deleted = self
            .repo
            .database
            .delete_entity_link(self.tag.id, source_id)
            .await?;

        if deleted {
            self.tag.update(|tag| tag.profile.delete_source(source_id));
        }

        Ok(deleted)
    }

    pub async fn delete_sources<S>(&self, sources: &[S]) -> Result<()>
    where
        S: AsRef<str>,
    {
        let ids = self
            .repo
            .entity(self.tag.id)
            .delete_sources(sources)
            .await?;

        self.tag.update(|tag| tag.profile.delete_sources(&ids));

        Ok(())
    }

    pub async fn set_description(
        &self,
        description: Description,
    ) -> Result<String> {
        let description: String = description.into();

        self.repo
            .database
            .update_entity_description(self.tag.id, &description)
            .await?
            .found("tag", self.tag.id)?;

        self.tag
            .update(|tag| tag.profile.description.clone_from(&description));

        Ok(description)
    }

    pub async fn set_name(&self, new_name: Name) -> Result<ProfileName> {
        let names = self
            .repo
            .entity(self.tag.id)
            .set_name(new_name, "tag", &self.repo.search.indices.tag)
            .await?;

        self.tag.update(|tag| tag.profile.set_names(&names));

        Ok(names)
    }
}
