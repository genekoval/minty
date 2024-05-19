use super::Repo;

use crate::{error::Found, search::Index, Error, Result};

use minty::{text::Name, ProfileName, Source, Url, Uuid};

pub struct Entity<'a> {
    repo: &'a Repo,
    id: Uuid,
}

impl<'a> Entity<'a> {
    pub(super) fn new(repo: &'a Repo, id: Uuid) -> Self {
        Self { repo, id }
    }

    pub async fn add_alias(
        &self,
        alias: Name,
        entity: &'static str,
        index: &Index,
    ) -> Result<ProfileName> {
        let alias = alias.as_ref();
        let mut tx = self.repo.database.begin().await?;

        let names = tx
            .create_entity_alias(self.id, alias)
            .await?
            .found(entity, self.id)?;
        self.repo
            .search
            .add_entity_alias(index, self.id, alias)
            .await?;

        tx.commit().await?;
        Ok(names.into())
    }

    pub async fn add_link(
        &self,
        entity: &'static str,
        url: &Url,
    ) -> Result<Source> {
        let source = self.repo.links().add(url).await?;

        self.repo
            .database
            .create_entity_link(self.id, source.id)
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "entity_link_profile_id_fkey" => {
                            Some(Error::NotFound {
                                entity,
                                id: self.id,
                            })
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?;

        Ok(source)
    }

    pub async fn delete(
        &self,
        entity: &'static str,
        index: &Index,
    ) -> Result<()> {
        let mut tx = self.repo.database.begin().await?;

        tx.delete_entity(self.id).await?.found(entity, self.id)?;
        index.delete_doc(self.id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_alias(
        &self,
        alias: &str,
        entity: &'static str,
        index: &Index,
    ) -> Result<ProfileName> {
        let mut tx = self.repo.database.begin().await?;

        let names = tx
            .delete_entity_alias(self.id, alias)
            .await?
            .found(entity, self.id)?;
        self.repo
            .search
            .delete_entity_alias(index, self.id, alias)
            .await?;

        tx.commit().await?;
        Ok(names.into())
    }

    pub async fn delete_sources<S>(&self, sources: &[S]) -> Result<()>
    where
        S: AsRef<str>,
    {
        let ids: Vec<i64> = self
            .repo
            .database
            .read_entity_sources(self.id)
            .await?
            .into_iter()
            .map(Into::<Source>::into)
            .filter(|existing| {
                let host = existing.url.host_str().unwrap();

                for source in sources.iter().map(AsRef::<str>::as_ref) {
                    match Url::parse(source).ok() {
                        Some(url) => {
                            if url == existing.url {
                                return true;
                            }
                        }
                        None => {
                            if source == host {
                                return true;
                            }
                        }
                    }
                }

                false
            })
            .map(|source| source.id)
            .collect();

        for source_id in ids {
            self.repo
                .database
                .delete_entity_link(self.id, source_id)
                .await?;
        }

        Ok(())
    }

    pub async fn set_name(
        &self,
        new_name: Name,
        entity: &'static str,
        index: &Index,
    ) -> Result<ProfileName> {
        let new_name = new_name.as_ref();
        let mut tx = self.repo.database.begin().await?;

        let update = tx
            .update_entity_name(self.id, new_name)
            .await?
            .found(entity, self.id)?;

        if let Some(old_name) = update.old_name {
            self.repo
                .search
                .update_entity_name(self.id, &old_name, new_name, index)
                .await?;
        }

        tx.commit().await?;
        Ok(update.names.into())
    }
}
