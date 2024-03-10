use crate::{
    conf::RepoConfig,
    db::{self, Database},
    error::{Error, Result},
    ico::Favicons,
    obj::Bucket,
    search::Search,
    About,
};

use bytes::Bytes;
use futures::TryStream;
use log::info;
use minty::model::*;
use std::{error, path::Path, result};

pub struct Repo {
    about: About,
    bucket: Bucket,
    database: Database,
    db_support: pgtools::Database,
    favicons: Favicons,
    search: Search,
}

impl Repo {
    pub async fn new(
        RepoConfig {
            version,
            objects,
            database,
            search,
        }: RepoConfig<'_>,
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

        let bucket = Bucket::new(objects).await?;
        let favicons = Favicons::new(bucket.clone());

        Ok(Self {
            about: About { version },
            bucket,
            database: Database::new(pool),
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
            favicons,
            search: Search::new(search)?,
        })
    }

    pub fn about(&self) -> &About {
        &self.about
    }

    pub async fn prepare(&self) -> result::Result<(), String> {
        self.db_support.check_schema_version().await
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
        self.database.prune().await?;

        let mut tx = self.database.begin().await?;

        let objects: Vec<Uuid> = tx
            .prune_objects()
            .await?
            .into_iter()
            .map(|row| row.0)
            .collect();

        let result = self.bucket.remove_objects(&objects).await?;

        tx.commit().await?;

        info!(
            "Removed {} {} freeing {}",
            result.objects_removed,
            match result.objects_removed {
                1 => "object",
                _ => "objects",
            },
            bytesize::to_string(result.space_freed, true),
        );

        Ok(())
    }

    pub async fn reset(&self) -> result::Result<(), String> {
        self.db_support.reset().await
    }

    pub async fn restore(&self, path: &Path) -> result::Result<(), String> {
        self.db_support.restore(path).await
    }

    pub async fn create_indices(&self) -> Result<()> {
        self.search.delete_indices().await?;
        self.search.create_indices().await
    }

    pub async fn add_comment(
        &self,
        post_id: Uuid,
        content: &str,
    ) -> Result<CommentData> {
        Ok(self.database.create_comment(post_id, content).await?.into())
    }

    pub async fn add_object<S>(&self, stream: S) -> Result<ObjectPreview>
    where
        S: TryStream + Send + 'static,
        S::Error: Into<Box<dyn error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        let object = self.bucket.add_object_stream(stream).await?;
        let preview = self.generate_preview(&object).await;

        self.database
            .create_object(object.id, preview, None)
            .await?;

        Ok(ObjectPreview {
            id: object.id,
            preview_id: preview,
            r#type: object.r#type,
            subtype: object.subtype,
        })
    }

    pub async fn add_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
        destination: Option<Uuid>,
    ) -> Result<DateTime> {
        let mut tx = self.database.begin().await?;

        let modified = tx
            .create_post_objects(post_id, objects, destination)
            .await?
            .0;
        self.search.update_post_modified(post_id, modified).await?;

        tx.commit().await?;
        Ok(modified)
    }

    pub async fn add_post_tag(
        &self,
        post_id: Uuid,
        tag_id: Uuid,
    ) -> Result<()> {
        let mut tx = self.database.begin().await?;

        tx.create_post_tag(post_id, tag_id).await?;
        self.search.add_post_tag(post_id, tag_id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn add_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        self.database.create_related_post(post_id, related).await?;
        Ok(())
    }

    pub async fn add_reply(
        &self,
        parent_id: Uuid,
        content: &str,
    ) -> Result<CommentData> {
        Ok(self.database.create_reply(parent_id, content).await?.into())
    }

    pub async fn add_tag(&self, name: &str) -> Result<Uuid> {
        let mut tx = self.database.begin().await?;

        let id = tx.create_tag(name).await?.0;
        self.search.add_tag_alias(id, name).await?;

        tx.commit().await?;
        Ok(id)
    }

    async fn add_site(&self, scheme: &str, host: &str) -> Result<i64> {
        let icon = self.favicons.download_icon(scheme, host).await;
        let site = self.database.create_site(scheme, host, icon).await?;

        Ok(site.site_id)
    }

    async fn add_source(&self, url: &Url) -> Result<Source> {
        const HOST_PREFIX: &str = "www.";

        let Some(host) = url.host_str() else {
            return Err(Error::InvalidInput(
                "expected a host in the source URL".into(),
            ));
        };

        let host = host.strip_prefix(HOST_PREFIX).unwrap_or(host);
        let scheme = url.scheme();

        let site = match self.database.read_site(scheme, host).await? {
            Some((site,)) => site,
            None => self.add_site(scheme, host).await?,
        };

        let mut resource = String::from(url.path());

        if let Some(query) = url.query() {
            if !query.is_empty() {
                resource = format!("{resource}?{query}");
            }
        }

        if let Some(fragment) = url.fragment() {
            if !fragment.is_empty() {
                resource = format!("{resource}#{fragment}");
            }
        }

        Ok(self.database.create_source(site, &resource).await?.into())
    }

    pub async fn add_tag_alias(
        &self,
        tag_id: Uuid,
        alias: &str,
    ) -> Result<TagName> {
        let mut tx = self.database.begin().await?;

        let names = tx.create_tag_alias(tag_id, alias).await?;
        self.search.add_tag_alias(tag_id, alias).await?;

        tx.commit().await?;
        Ok(names.into())
    }

    pub async fn add_tag_source(
        &self,
        tag_id: Uuid,
        url: &Url,
    ) -> Result<Source> {
        let source = self.add_source(url).await?;
        self.database.create_tag_source(tag_id, source.id).await?;

        Ok(source)
    }

    pub async fn create_post(&self, parts: &PostParts) -> Result<Uuid> {
        let mut tx = self.database.begin().await?;

        let post = tx
            .create_post(
                parts.title.as_deref().unwrap_or(""),
                parts.description.as_deref().unwrap_or(""),
                parts.visibility.map(db::Visibility::from_minty),
                parts.objects.as_deref().unwrap_or(&[]),
                parts.posts.as_deref().unwrap_or(&[]),
                parts.tags.as_deref().unwrap_or(&[]),
            )
            .await?;

        self.search.add_post(&post).await?;

        tx.commit().await?;
        Ok(post.id)
    }

    pub async fn delete_comment(
        &self,
        id: Uuid,
        recursive: bool,
    ) -> Result<bool> {
        Ok(self.database.delete_comment(id, recursive).await?)
    }

    pub async fn delete_post(&self, id: Uuid) -> Result<()> {
        let mut tx = self.database.begin().await?;

        tx.delete_post(id).await?;
        self.search.delete_post(id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_post_objects(
        &self,
        post_id: Uuid,
        objects: &[Uuid],
    ) -> Result<DateTime> {
        let mut tx = self.database.begin().await?;

        let modified = tx.delete_post_objects(post_id, objects).await?.0;
        self.search.update_post_modified(post_id, modified).await?;

        tx.commit().await?;
        Ok(modified)
    }

    pub async fn delete_post_tag(
        &self,
        post_id: Uuid,
        tag_id: Uuid,
    ) -> Result<()> {
        let mut tx = self.database.begin().await?;

        tx.delete_post_tag(post_id, tag_id).await?;
        self.search.remove_post_tag(post_id, tag_id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        self.database.delete_related_post(post_id, related).await?;
        Ok(())
    }

    pub async fn delete_tag(&self, id: Uuid) -> Result<()> {
        let mut tx = self.database.begin().await?;

        tx.delete_tag(id).await?;
        self.search.delete_tag(id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_tag_alias(
        &self,
        tag_id: Uuid,
        alias: &str,
    ) -> Result<TagName> {
        let mut tx = self.database.begin().await?;

        let names = tx.delete_tag_alias(tag_id, alias).await?;
        self.search.delete_tag_alias(tag_id, alias).await?;

        tx.commit().await?;
        Ok(names.into())
    }

    pub async fn delete_tag_source(
        &self,
        tag_id: Uuid,
        source_id: i64,
    ) -> Result<()> {
        self.database.delete_tag_source(tag_id, source_id).await?;
        Ok(())
    }

    pub async fn delete_tag_sources<S>(
        &self,
        tag_id: Uuid,
        sources: &[S],
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        let ids: Vec<i64> = self
            .database
            .read_tag_sources(tag_id)
            .await?
            .into_iter()
            .filter(|existing| {
                let existing_host = &existing.site.host;
                let existing_url = existing.url();

                for source in sources {
                    let source = source.as_ref();

                    match Url::parse(source).ok() {
                        Some(url) => {
                            if url == existing_url {
                                return true;
                            }
                        }
                        None => {
                            if source == existing_host {
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
            self.database.delete_tag_source(tag_id, source_id).await?;
        }

        Ok(())
    }

    pub async fn get_comment(&self, id: Uuid) -> Result<Comment> {
        Ok(self
            .database
            .read_comment(id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "comment with ID '{id}' does not exist"
                ))
            })?
            .into())
    }

    pub async fn get_comments(
        &self,
        post_id: Uuid,
    ) -> Result<Vec<CommentData>> {
        Ok(self
            .database
            .read_comments(post_id)
            .await?
            .into_iter()
            .map(|comment| comment.into())
            .collect())
    }

    pub async fn get_object(&self, id: Uuid) -> Result<Object> {
        let object = self.database.read_object(id).await?.ok_or_else(|| {
            Error::NotFound(format!("object with ID '{id}' does not exist"))
        })?;

        let posts = self.database.read_object_posts(id).await?;
        let metadata = self.bucket.get_object(id).await?;

        Ok(Object {
            id,
            hash: metadata.hash,
            size: metadata.size,
            r#type: metadata.r#type,
            subtype: metadata.subtype,
            added: metadata.added,
            preview_id: object.preview_id,
            posts: self.build_posts(posts).await?,
        })
    }

    pub async fn get_object_preview_errors(&self) -> Result<Vec<ObjectError>> {
        Ok(self
            .database
            .read_object_preview_errors()
            .await?
            .into_iter()
            .map(|e| e.into())
            .collect())
    }

    pub async fn get_post(&self, id: Uuid) -> Result<Post> {
        let post = self.database.read_post(id).await?.ok_or_else(|| {
            Error::NotFound(format!("post with ID '{id}' does not exist"))
        })?;

        Ok(Post {
            id: post.id,
            title: post.title,
            description: post.description,
            visibility: post.visibility.into(),
            created: post.created,
            modified: post.modified,
            objects: self.bucket.get_objects(post.objects).await?,
            posts: self.build_posts(post.posts).await?,
            tags: post.tags.into_iter().map(|tag| tag.into()).collect(),
            comment_count: post.comment_count,
        })
    }

    pub async fn get_posts(
        &self,
        query: &PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        let results = self.search.find_posts(query).await?;
        let posts = self.database.read_posts(&results.hits).await?;
        let posts = self.build_posts(posts).await?;

        Ok(SearchResult {
            total: results.total,
            hits: posts,
        })
    }

    async fn build_posts(
        &self,
        posts: Vec<db::PostPreview>,
    ) -> Result<Vec<PostPreview>> {
        let objects = posts
            .iter()
            .filter_map(|post| post.preview.clone())
            .collect();

        let mut objects = self.bucket.get_objects(objects).await?.into_iter();

        let posts = posts
            .into_iter()
            .map(|post| PostPreview {
                id: post.id,
                title: post.title,
                preview: if post.preview.is_some() {
                    objects.next()
                } else {
                    None
                },
                comment_count: post.comment_count,
                object_count: post.object_count,
                created: post.created,
            })
            .collect();

        Ok(posts)
    }

    pub async fn get_tag(&self, id: Uuid) -> Result<Tag> {
        let mut tag: Tag = self
            .database
            .read_tag(id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("tag with ID '{id}' does not exist"))
            })?
            .into();

        tag.sources = self
            .database
            .read_tag_sources(id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(tag)
    }

    pub async fn get_tags(
        &self,
        query: &TagQuery,
    ) -> Result<SearchResult<TagPreview>> {
        let results = self.search.find_tags(query).await?;
        let tags = self
            .database
            .read_tag_previews(&results.hits)
            .await?
            .into_iter()
            .map(|tag| tag.into())
            .collect();

        Ok(SearchResult {
            total: results.total,
            hits: tags,
        })
    }

    pub async fn publish_post(&self, post_id: Uuid) -> Result<()> {
        let mut tx = self.database.begin().await?;

        let timestamp = tx.publish_post(post_id).await?.0;
        self.search.publish_post(post_id, timestamp).await?;

        tx.commit().await?;
        Ok(())
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
        self.database.update_comment(comment_id, content).await?;
        Ok(content.into())
    }

    pub async fn set_post_description(
        &self,
        post_id: Uuid,
        description: &str,
    ) -> Result<Modification<String>> {
        let mut tx = self.database.begin().await?;

        let modified = tx
            .update_post_description(post_id, description)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "post with ID '{post_id}' does not exist"
                ))
            })?
            .0;

        self.search
            .update_post_description(post_id, description, modified)
            .await?;

        tx.commit().await?;
        Ok(Modification {
            date_modified: modified,
            new_value: description.into(),
        })
    }

    pub async fn set_post_title(
        &self,
        post_id: Uuid,
        title: &str,
    ) -> Result<Modification<String>> {
        let mut tx = self.database.begin().await?;

        let modified = tx
            .update_post_title(post_id, title)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "post with ID '{post_id}' does not exist"
                ))
            })?
            .0;

        self.search
            .update_post_title(post_id, title, modified)
            .await?;

        tx.commit().await?;
        Ok(Modification {
            date_modified: modified,
            new_value: title.into(),
        })
    }

    pub async fn set_tag_description(
        &self,
        tag_id: Uuid,
        description: &str,
    ) -> Result<String> {
        let found = self
            .database
            .update_tag_description(tag_id, description)
            .await?;

        if found {
            Ok(description.into())
        } else {
            Err(Error::NotFound(format!(
                "tag with ID '{tag_id}' does not exist"
            )))
        }
    }

    pub async fn set_tag_name(
        &self,
        tag_id: Uuid,
        new_name: &str,
    ) -> Result<TagName> {
        let mut tx = self.database.begin().await?;

        let update = tx.update_tag_name(tag_id, new_name).await?;

        if let Some(old_name) = update.old_name {
            self.search
                .update_tag_name(tag_id, &old_name, new_name)
                .await?;
        }

        tx.commit().await?;
        Ok(update.names.into())
    }

    async fn generate_preview(&self, object: &fstore::Object) -> Option<Uuid> {
        todo!()
    }
}
