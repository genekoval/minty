use crate::{conf::RepoConfig, db, error::Result, About};

use minty::model::*;
use std::{path::Path, result};

pub struct Repo {
    about: About,
    database: db::Database,
    db_support: pgtools::Database,
}

impl Repo {
    pub async fn new(
        RepoConfig { version, database }: RepoConfig<'_>,
    ) -> result::Result<Self, String> {
        let mut pool = db::PoolOptions::new();

        if let Some(max_connections) = database.max_connections {
            pool = pool.max_connections(max_connections);
        }

        let pool = pool
            .connect(database.connection.as_url().as_str())
            .await
            .map_err(|err| {
                format!("failed to establish database connection: {err}")
            })?;

        Ok(Self {
            about: About { version },
            database: db::Database::new(pool),
            db_support: pgtools::Database::new(
                version.number,
                pgtools::Options {
                    connection: &database.connection,
                    psql: &database.psql,
                    pg_dump: &database.pg_dump,
                    pg_restore: &database.pg_restore,
                    sql_directory: &database.sql_directory,
                },
            )?,
        })
    }

    pub async fn shutdown(&self) {
        self.database.close().await;
    }

    pub async fn dump(&self, path: &Path) -> result::Result<(), String> {
        self.db_support.dump(path).await
    }

    pub async fn init(&self) -> result::Result<(), String> {
        self.db_support.init().await
    }

    pub async fn migrate(&self) -> result::Result<(), String> {
        self.db_support.migrate().await
    }

    pub async fn prune(&self) -> Result<()> {
        todo!()
    }

    pub async fn reset(&self) -> result::Result<(), String> {
        self.db_support.reset().await
    }

    pub async fn restore(&self, path: &Path) -> result::Result<(), String> {
        self.db_support.restore(path).await
    }

    pub async fn about(&self) -> &About {
        &self.about
    }

    pub async fn add_comment(
        &self,
        post_id: Uuid,
        content: &str,
    ) -> Result<CommentData> {
        todo!()
    }

    pub async fn add_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
        destination: Option<Uuid>,
    ) -> Result<DateTime> {
        todo!()
    }

    pub async fn add_post_tag(
        &self,
        post_id: Uuid,
        tag_id: Uuid,
    ) -> Result<()> {
        todo!()
    }

    pub async fn add_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        todo!()
    }

    pub async fn add_reply(
        &self,
        parent_id: Uuid,
        content: &str,
    ) -> Result<CommentData> {
        todo!()
    }

    pub async fn add_tag(&self, name: &str) -> Result<Uuid> {
        todo!()
    }

    pub async fn add_tag_alias(
        &self,
        tag_id: Uuid,
        alias: &str,
    ) -> Result<TagName> {
        todo!()
    }

    pub async fn add_tag_source(
        &self,
        tag_id: Uuid,
        url: &Url,
    ) -> Result<Source> {
        todo!()
    }

    pub async fn create_post(&self, post_id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn create_post_draft(&self) -> Result<Uuid> {
        todo!()
    }

    pub async fn delete_comment(
        &self,
        id: Uuid,
        recursive: bool,
    ) -> Result<bool> {
        todo!()
    }

    pub async fn delete_post(&self, id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn delete_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime> {
        todo!()
    }

    pub async fn delete_post_tag(
        &self,
        post_id: Uuid,
        tag_id: Uuid,
    ) -> Result<()> {
        todo!()
    }

    pub async fn delete_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        todo!()
    }

    pub async fn delete_tag(&self, id: Uuid) -> Result<()> {
        todo!()
    }

    pub async fn delete_tag_alias(
        &self,
        tag_id: Uuid,
        alias: &str,
    ) -> Result<TagName> {
        todo!()
    }

    pub async fn delete_tag_source(
        &self,
        tag_id: Uuid,
        source_id: i64,
    ) -> Result<()> {
        todo!()
    }

    pub async fn get_comment(&self, id: Uuid) -> Result<Comment> {
        todo!()
    }

    pub async fn get_comments(
        &self,
        post_id: Uuid,
    ) -> Result<Vec<CommentData>> {
        todo!()
    }

    pub async fn get_object(&self, id: Uuid) -> Result<Object> {
        todo!()
    }

    pub async fn get_object_preview_errors(&self) -> Vec<ObjectError> {
        todo!()
    }

    pub async fn get_post(&self, id: Uuid) -> Result<Post> {
        todo!()
    }

    pub async fn get_posts(
        &self,
        query: &PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        todo!()
    }

    pub async fn get_tag(&self, id: Uuid) -> Result<Tag> {
        todo!()
    }

    pub async fn get_tags(
        &self,
        query: &TagQuery,
    ) -> Result<SearchResult<TagPreview>> {
        todo!()
    }

    pub async fn regenerate_preview(
        &self,
        object_id: Uuid,
    ) -> Result<Option<Uuid>> {
        todo!()
    }

    pub async fn set_comment_content(
        &self,
        comment_id: Uuid,
        content: &str,
    ) -> Result<String> {
        todo!()
    }

    pub async fn set_post_description(
        &self,
        post_id: Uuid,
        description: &str,
    ) -> Result<Modification<String>> {
        todo!()
    }

    pub async fn set_post_title(
        &self,
        post_id: Uuid,
        title: &str,
    ) -> Result<Modification<String>> {
        todo!()
    }

    pub async fn set_tag_description(
        &self,
        tag_id: Uuid,
        description: &str,
    ) -> Result<String> {
        todo!()
    }

    pub async fn set_tag_name(
        &self,
        tag_id: Uuid,
        new_name: &str,
    ) -> Result<TagName> {
        todo!()
    }
}
