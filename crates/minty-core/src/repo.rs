use crate::{
    auth::Auth,
    comment,
    conf::RepoConfig,
    db::{self, Database, Id},
    error::{Error, Found, Result},
    ico::Favicons,
    obj::Bucket,
    preview,
    search::{Index, Search},
    task::Task,
    About,
};

use bytes::Bytes;
use fstore::RemoveResult;
use futures::{stream::BoxStream, Stream, StreamExt, TryStream};
use log::error;
use minty::{model::*, text};
use serde::Serialize;
use std::{error, io, path::Path, result, sync::Arc};
use tokio::{
    sync::Semaphore,
    task::{self, JoinHandle},
};
use tokio_util::task::TaskTracker;

pub struct Repo {
    auth: Auth,
    bucket: Bucket,
    database: Database,
    db_support: pgtools::Database,
    favicons: Favicons,
    search: Search,
}

impl Repo {
    pub async fn new(config: &RepoConfig) -> result::Result<Self, String> {
        let mut pool = db::PoolOptions::new();

        if let Some(max_connections) = config.database.max_connections {
            pool = pool.max_connections(max_connections);
        }

        let pool = pool
            .connect(config.database.connection.as_url().as_str())
            .await
            .map_err(|err| {
                format!("failed to establish database connection: {err}")
            })?;

        let db_support = pgtools::Database::new(
            crate::VERSION,
            pgtools::Options {
                connection: &config.database.connection,
                psql: &config.database.psql,
                pg_dump: &config.database.pg_dump,
                pg_restore: &config.database.pg_restore,
                sql_directory: &config.database.sql_directory,
            },
        )?;

        let database = Database::new(pool);
        let bucket = Bucket::new(&config.objects).await?;
        let favicons = Favicons::new(bucket.clone());

        Ok(Self {
            bucket,
            database,
            db_support,
            favicons,
            auth: Auth::new(),
            search: Search::new(&config.search)?,
        })
    }

    pub fn about(&self) -> About {
        About {
            version: crate::VERSION,
        }
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

    pub async fn prune(&self) -> Result<RemoveResult> {
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
        Ok(result)
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

    pub async fn import(&self, data: &export::Data) -> Result<()> {
        let mut objects: Vec<_> = data
            .posts
            .iter()
            .flat_map(|post| post.objects.as_slice())
            .copied()
            .collect();
        objects.sort_unstable();
        objects.dedup();

        let objects = self.bucket.get_objects(&objects).await?;

        for object in objects {
            self.add_object(object).await?;
        }
        self.database.import(sqlx::types::Json(data)).await?;

        self.import_profile(&data.tags).await?;
        self.import_profile(&data.users).await?;

        Ok(())
    }

    pub async fn import_profile<P>(&self, entities: &[P]) -> Result<()>
    where
        P: export::Profile,
    {
        for entity in entities {
            for export::Source { url, icon } in &entity.profile().sources {
                let scheme = url.scheme();
                let host = url.host_str().unwrap();
                let resource = &url[url::Position::BeforePath..];

                let site = match self.database.read_site(scheme, host).await? {
                    (Some(site),) => site,
                    (None,) => {
                        self.database
                            .create_site(scheme, host, *icon)
                            .await?
                            .site_id
                    }
                };

                let source =
                    self.database.create_source(site, resource).await?;

                self.database
                    .create_entity_link(entity.id(), source.id)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn export(&self) -> Result<export::Data> {
        let (sqlx::types::Json(data),) = self.database.export().await?;
        Ok(data)
    }

    pub async fn add_comment(
        &self,
        user_id: Uuid,
        post_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        Ok(self
            .database
            .create_comment(user_id, post_id, content.as_ref())
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "post_comment_post_id_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: post_id,
                        }),
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?
            .into())
    }

    async fn add_entity_alias(
        &self,
        profile_id: Uuid,
        alias: text::Name,
        entity: &'static str,
        index: &Index,
    ) -> Result<ProfileName> {
        let alias = alias.as_ref();
        let mut tx = self.database.begin().await?;

        let names = tx
            .create_entity_alias(profile_id, alias)
            .await?
            .found(entity, profile_id)?;
        self.search
            .add_entity_alias(index, profile_id, alias)
            .await?;

        tx.commit().await?;
        Ok(names.into())
    }

    async fn add_entity_link(
        &self,
        entity: &'static str,
        profile_id: Uuid,
        url: &Url,
    ) -> Result<Source> {
        let source = self.add_source(url).await?;

        self.database
            .create_entity_link(profile_id, source.id)
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "entity_link_profile_id_fkey" => {
                            Some(Error::NotFound {
                                entity,
                                id: profile_id,
                            })
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?;

        Ok(source)
    }

    async fn add_object(
        &self,
        object: fstore::Object,
    ) -> Result<ObjectPreview> {
        let result = preview::generate_preview(&self.bucket, &object).await;
        let preview = result.as_ref().ok().cloned().flatten();

        self.database
            .create_object(object.id, preview, None)
            .await?;

        if let Err(preview_error) = result {
            if let Err(err) = self
                .database
                .create_object_preview_error(object.id, &preview_error)
                .await
            {
                error!(
                    "Failed to write object preview error to database: {err}; \
                    error for object '{}': {preview_error}",
                    object.id
                );
            }
        }

        Ok(ObjectPreview {
            id: object.id,
            preview_id: preview,
            r#type: object.r#type,
            subtype: object.subtype,
        })
    }

    pub async fn add_object_stream<S>(&self, stream: S) -> Result<ObjectPreview>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        let object = self.bucket.add_object_stream(stream).await?;
        self.add_object(object).await
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

        tx.create_post_tag(post_id, tag_id).await.map_err(|err| {
            err.as_database_error()
                .and_then(|e| e.constraint())
                .and_then(|constraint| match constraint {
                    "post_tag_post_id_fkey" => Some(Error::NotFound {
                        entity: "post",
                        id: post_id,
                    }),
                    "post_tag_tag_id_fkey" => Some(Error::NotFound {
                        entity: "tag",
                        id: tag_id,
                    }),
                    _ => None,
                })
                .unwrap_or_else(|| err.into())
        })?;
        self.search.add_post_tag(post_id, tag_id).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn add_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<()> {
        if post_id == related {
            return Err(Error::InvalidInput(
                "post cannot be related to itself".into(),
            ));
        }

        self.database
            .create_related_post(post_id, related)
            .await
            .map_err(|err| {
                err.as_database_error()
                    .and_then(|e| e.constraint())
                    .and_then(|constraint| match constraint {
                        "related_post_post_id_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: post_id,
                        }),
                        "related_post_related_fkey" => Some(Error::NotFound {
                            entity: "post",
                            id: related,
                        }),
                        _ => None,
                    })
                    .unwrap_or_else(|| err.into())
            })?;

        Ok(())
    }

    pub async fn add_reply(
        &self,
        user_id: Uuid,
        parent_id: Uuid,
        content: text::Comment,
    ) -> Result<CommentData> {
        Ok(self
            .database
            .create_reply(user_id, parent_id, content.as_ref())
            .await?
            .found("comment", parent_id)?
            .into())
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
            (Some(site),) => site,
            (None,) => self.add_site(scheme, host).await?,
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

    pub async fn add_tag(
        &self,
        name: text::Name,
        creator: Uuid,
    ) -> Result<Uuid> {
        let name = name.as_ref();
        let mut tx = self.database.begin().await?;

        let id = tx.create_tag(name, creator).await?.0;
        self.search.add_tag_alias(id, name).await?;

        tx.commit().await?;
        Ok(id)
    }

    pub async fn add_tag_alias(
        &self,
        tag_id: Uuid,
        alias: text::Name,
    ) -> Result<ProfileName> {
        self.add_entity_alias(tag_id, alias, "tag", &self.search.indices.tag)
            .await
    }

    pub async fn add_tag_source(
        &self,
        tag_id: Uuid,
        url: &Url,
    ) -> Result<Source> {
        self.add_entity_link("tag", tag_id, url).await
    }

    pub async fn add_user(&self, info: SignUp) -> Result<Uuid> {
        let name = info.username.as_ref();
        let email = info.email.as_ref();
        let password = self.auth.hash_password(info.password)?;

        let mut tx = self.database.begin().await?;

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

        self.search.add_user_alias(id, name).await?;

        tx.commit().await?;
        Ok(id)
    }

    pub async fn add_user_alias(
        &self,
        user_id: Uuid,
        alias: text::Name,
    ) -> Result<ProfileName> {
        self.add_entity_alias(user_id, alias, "user", &self.search.indices.user)
            .await
    }

    pub async fn add_user_source(
        &self,
        user_id: Uuid,
        url: &Url,
    ) -> Result<Source> {
        self.add_entity_link("user", user_id, url).await
    }

    pub async fn authenticate(&self, login: &Login) -> Result<Uuid> {
        let Some(user) = self.database.read_user_password(&login.email).await?
        else {
            return Err(Error::Unauthenticated);
        };

        let authenticated =
            self.auth.verify_password(&login.password, &user.password)?;

        if authenticated {
            Ok(user.user_id)
        } else {
            Err(Error::Unauthenticated)
        }
    }

    pub async fn create_post(
        &self,
        user: Uuid,
        parts: &PostParts,
    ) -> Result<Uuid> {
        let mut tx = self.database.begin().await?;

        let post = tx
            .create_post(
                user,
                parts.title.as_ref().map(|t| t.as_ref()).unwrap_or(""),
                parts.description.as_ref().map(|d| d.as_ref()).unwrap_or(""),
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

    async fn delete_entity(
        &self,
        profile_id: Uuid,
        entity: &'static str,
        index: &Index,
    ) -> Result<()> {
        let mut tx = self.database.begin().await?;

        tx.delete_entity(profile_id)
            .await?
            .found(entity, profile_id)?;
        index.delete_doc(profile_id).await?;

        tx.commit().await?;
        Ok(())
    }

    async fn delete_entity_alias(
        &self,
        profile_id: Uuid,
        alias: &str,
        entity: &'static str,
        index: &Index,
    ) -> Result<ProfileName> {
        let mut tx = self.database.begin().await?;

        let names = tx
            .delete_entity_alias(profile_id, alias)
            .await?
            .found(entity, profile_id)?;
        self.search
            .delete_entity_alias(index, profile_id, alias)
            .await?;

        tx.commit().await?;
        Ok(names.into())
    }

    async fn delete_entity_sources<S>(
        &self,
        profile_id: Uuid,
        sources: &[S],
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        let ids: Vec<i64> = self
            .database
            .read_entity_sources(profile_id)
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
            self.database
                .delete_entity_link(profile_id, source_id)
                .await?;
        }

        Ok(())
    }

    pub async fn delete_post(&self, id: Uuid) -> Result<()> {
        let mut tx = self.database.begin().await?;

        tx.delete_post(id).await?.found("post", id)?;
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
    ) -> Result<bool> {
        let mut tx = self.database.begin().await?;

        let found = tx.delete_post_tag(post_id, tag_id).await?;
        if found {
            self.search.remove_post_tag(post_id, tag_id).await?;
        }

        tx.commit().await?;
        Ok(found)
    }

    pub async fn delete_related_post(
        &self,
        post_id: Uuid,
        related: Uuid,
    ) -> Result<bool> {
        Ok(self.database.delete_related_post(post_id, related).await?)
    }

    pub async fn delete_tag(&self, id: Uuid) -> Result<()> {
        self.delete_entity(id, "tag", &self.search.indices.tag)
            .await
    }

    pub async fn delete_tag_alias(
        &self,
        tag_id: Uuid,
        alias: &str,
    ) -> Result<ProfileName> {
        self.delete_entity_alias(tag_id, alias, "tag", &self.search.indices.tag)
            .await
    }

    pub async fn delete_tag_source(
        &self,
        tag_id: Uuid,
        source_id: i64,
    ) -> Result<bool> {
        Ok(self.database.delete_entity_link(tag_id, source_id).await?)
    }

    pub async fn delete_tag_sources<S>(
        &self,
        tag_id: Uuid,
        sources: &[S],
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.delete_entity_sources(tag_id, sources).await
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        self.delete_entity(id, "user", &self.search.indices.user)
            .await
    }

    pub async fn delete_user_alias(
        &self,
        user_id: Uuid,
        alias: &str,
    ) -> Result<ProfileName> {
        self.delete_entity_alias(
            user_id,
            alias,
            "user",
            &self.search.indices.user,
        )
        .await
    }

    pub async fn delete_user_source(
        &self,
        user_id: Uuid,
        source_id: i64,
    ) -> Result<bool> {
        Ok(self.database.delete_entity_link(user_id, source_id).await?)
    }

    pub async fn delete_user_sources<S>(
        &self,
        user_id: Uuid,
        sources: &[S],
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.delete_entity_sources(user_id, sources).await
    }

    pub async fn get_comment(&self, id: Uuid) -> Result<Comment> {
        Ok(self
            .database
            .read_comment(id)
            .await?
            .found("comment", id)?
            .into())
    }

    pub async fn get_comments(
        &self,
        post_id: Uuid,
    ) -> Result<Vec<CommentData>> {
        let comments = self.database.read_comments(post_id).await?;
        Ok(comment::build_tree(comments))
    }

    pub async fn get_object(&self, id: Uuid) -> Result<Object> {
        let object =
            self.database.read_object(id).await?.found("object", id)?;

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

    pub async fn get_object_data(
        &self,
        id: Uuid,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)> {
        self.bucket.get_object_stream(id).await
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
        let post = self.database.read_post(id).await?.found("post", id)?;

        Ok(Post {
            id: post.id,
            poster: post.poster.map(Into::into),
            title: post.title,
            description: post.description,
            visibility: post.visibility.into(),
            created: post.created,
            modified: post.modified,
            objects: self.bucket.get_object_previews(post.objects).await?,
            posts: self.build_posts(post.posts).await?,
            tags: post.tags.into_iter().map(|tag| tag.into()).collect(),
            comment_count: post.comment_count,
        })
    }

    pub async fn get_posts(
        &self,
        user_id: Option<Uuid>,
        mut query: PostQuery,
    ) -> Result<SearchResult<PostPreview>> {
        if user_id.is_none() && query.visibility != Visibility::Public {
            return Err(Error::Unauthenticated);
        }

        if query.visibility == Visibility::Draft {
            query.poster = user_id;
        }

        let results = self.search.find_posts(&query).await?;
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

        let mut objects =
            self.bucket.get_object_previews(objects).await?.into_iter();

        let posts = posts
            .into_iter()
            .map(|post| PostPreview {
                id: post.id,
                poster: post.poster.map(Into::into),
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
        Ok(self.database.read_tag(id).await?.found("tag", id)?.into())
    }

    pub async fn get_tags(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<TagPreview>> {
        let results = self
            .search
            .find_entities(&self.search.indices.tag, query)
            .await?;
        let tags = self
            .database
            .read_tag_previews(&results.hits)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(SearchResult {
            total: results.total,
            hits: tags,
        })
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User> {
        Ok(self.database.read_user(id).await?.found("user", id)?.into())
    }

    pub async fn get_users(
        &self,
        query: &ProfileQuery,
    ) -> Result<SearchResult<UserPreview>> {
        let results = self
            .search
            .find_entities(&self.search.indices.user, query)
            .await?;
        let users = self
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

    pub async fn publish_post(&self, post_id: Uuid) -> Result<()> {
        let mut tx = self.database.begin().await?;

        let timestamp = tx.publish_post(post_id).await?.0;
        self.search.publish_post(post_id, timestamp).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn set_comment_content(
        &self,
        comment_id: Uuid,
        content: text::Comment,
    ) -> Result<String> {
        self.database
            .update_comment(comment_id, content.as_ref())
            .await?
            .found("comment", comment_id)?;

        Ok(content.into())
    }

    pub async fn set_entity_name(
        &self,
        profile_id: Uuid,
        new_name: text::Name,
        entity: &'static str,
        index: &Index,
    ) -> Result<ProfileName> {
        let new_name = new_name.as_ref();
        let mut tx = self.database.begin().await?;

        let update = tx
            .update_entity_name(profile_id, new_name)
            .await?
            .found(entity, profile_id)?;

        if let Some(old_name) = update.old_name {
            self.search
                .update_entity_name(profile_id, &old_name, new_name, index)
                .await?;
        }

        tx.commit().await?;
        Ok(update.names.into())
    }

    pub async fn set_post_description(
        &self,
        post_id: Uuid,
        description: text::Description,
    ) -> Result<Modification<String>> {
        let mut tx = self.database.begin().await?;

        let (modified,) = tx
            .update_post_description(post_id, description.as_ref())
            .await?
            .found("post", post_id)?;

        self.search
            .update_post_description(post_id, description.as_ref(), modified)
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
        title: text::PostTitle,
    ) -> Result<Modification<String>> {
        let mut tx = self.database.begin().await?;

        let (modified,) = tx
            .update_post_title(post_id, title.as_ref())
            .await?
            .found("post", post_id)?;

        self.search
            .update_post_title(post_id, title.as_ref(), modified)
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
        description: text::Description,
    ) -> Result<String> {
        self.database
            .update_entity_description(tag_id, description.as_ref())
            .await?
            .found("tag", tag_id)?;

        Ok(description.into())
    }

    pub async fn set_tag_name(
        &self,
        tag_id: Uuid,
        new_name: text::Name,
    ) -> Result<ProfileName> {
        self.set_entity_name(tag_id, new_name, "tag", &self.search.indices.tag)
            .await
    }

    pub async fn set_user_description(
        &self,
        user_id: Uuid,
        description: text::Description,
    ) -> Result<String> {
        self.database
            .update_entity_description(user_id, description.as_ref())
            .await?
            .found("user", user_id)?;

        Ok(description.into())
    }

    pub async fn set_user_email(
        &self,
        user_id: Uuid,
        email: text::Email,
    ) -> Result<()> {
        self.database
            .update_user_email(user_id, email.as_ref())
            .await?
            .found("user", user_id)?;

        Ok(())
    }

    pub async fn set_user_password(
        &self,
        user_id: Uuid,
        password: text::Password,
    ) -> Result<()> {
        let password = self.auth.hash_password(password)?;

        self.database
            .update_user_password(user_id, &password)
            .await?
            .found("user", user_id)?;

        Ok(())
    }

    pub async fn set_user_name(
        &self,
        user_id: Uuid,
        new_name: text::Name,
    ) -> Result<ProfileName> {
        self.set_entity_name(
            user_id,
            new_name,
            "user",
            &self.search.indices.user,
        )
        .await
    }

    pub async fn regenerate_preview(
        &self,
        object: Uuid,
    ) -> Result<Option<Uuid>> {
        let object = self.bucket.get_object(object).await?;

        match preview::generate_preview(&self.bucket, &object).await {
            Ok(preview) => {
                self.database
                    .update_object_preview(object.id, preview)
                    .await?;
                Ok(preview)
            }
            Err(message) => {
                self.database
                    .create_object_preview_error(object.id, &message)
                    .await?;
                Err(Error::Internal(message))
            }
        }
    }

    pub async fn regenerate_previews(
        self: &Arc<Self>,
        batch_size: usize,
        max_tasks: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total =
            self.database.read_object_total().await?.try_into().unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.clone();

        let handle = task::spawn(async move {
            repo.regenerate_previews_task(guard.task(), batch_size, max_tasks)
                .await
        });

        Ok((task, handle))
    }

    async fn regenerate_previews_task(
        self: Arc<Self>,
        task: Task,
        batch_size: usize,
        max_tasks: usize,
    ) -> Result<()> {
        let tracker = TaskTracker::new();
        let semaphore = Arc::new(Semaphore::new(max_tasks));
        let mut error: Option<Error> = None;
        let mut stream = self.database.stream_objects().chunks(batch_size);

        'stream: while let Some(chunk) = stream.next().await {
            let objects = match chunk
                .into_iter()
                .collect::<result::Result<Vec<_>, _>>()
                .map(|objects| {
                    objects
                        .into_iter()
                        .map(|object| object.id)
                        .collect::<Vec<_>>()
                }) {
                Ok(objects) => objects,
                Err(err) => {
                    error = Some(err.into());
                    break 'stream;
                }
            };

            let objects = match self.bucket.get_objects(&objects).await {
                Ok(objects) => objects,
                Err(err) => {
                    error = Some(err);
                    break 'stream;
                }
            };

            for object in objects {
                let permit = tokio::select! {
                    biased;

                    _ = task.cancelled() => {
                        break 'stream;
                    }
                    permit = semaphore.clone().acquire_owned() => {
                        permit.unwrap()
                    }
                };

                let repo = self.clone();
                let task = task.clone();

                tracker.spawn(async move {
                    repo.regenerate_previews_subtask(&task, &object).await;
                    task.increment();
                    drop(permit);
                });
            }
        }

        tracker.close();
        tracker.wait().await;

        match error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    async fn regenerate_previews_subtask(
        &self,
        task: &Task,
        object: &fstore::Object,
    ) {
        match preview::generate_preview(&self.bucket, object).await {
            Ok(preview) => {
                if let Err(err) = self
                    .database
                    .update_object_preview(object.id, preview)
                    .await
                {
                    error!(
                        "Failed to update object \
                        preview for {}: {err}",
                        object.id
                    );
                    task.error();
                }
            }
            Err(message) => {
                if let Err(err) = self
                    .database
                    .create_object_preview_error(object.id, &message)
                    .await
                {
                    error!(
                        "Failed to write object preview error \
                        for {}: {err}; preview error: {message}",
                        object.id
                    );
                }
                task.error();
            }
        }
    }

    async fn reindex<T>(
        &self,
        task: Task,
        index: &Index,
        batch_size: usize,
        stream: BoxStream<'_, sqlx::Result<T>>,
    ) -> Result<()>
    where
        T: Id + Serialize,
    {
        index.recreate().await?;

        let mut stream = stream.chunks(batch_size);

        while let Some(chunk) = stream.next().await {
            let items =
                chunk.into_iter().collect::<result::Result<Vec<_>, _>>()?;

            index.bulk_create(&items).await?;
            task.progress(items.len());
        }

        index.refresh().await?;

        Ok(())
    }

    pub async fn reindex_posts(
        self: &Arc<Self>,
        batch_size: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total = self.database.read_post_total().await?.try_into().unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.clone();

        let handle = task::spawn(async move {
            let index = &repo.search.indices.post;
            let stream = repo.database.read_post_search();

            repo.reindex(guard.task(), index, batch_size, stream).await
        });

        Ok((task, handle))
    }

    pub async fn reindex_tags(
        self: &Arc<Self>,
        batch_size: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total = self.database.read_tag_total().await?.try_into().unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.clone();

        let handle = task::spawn(async move {
            let index = &repo.search.indices.tag;
            let stream = repo.database.read_tag_search();

            repo.reindex(guard.task(), index, batch_size, stream).await
        });

        Ok((task, handle))
    }

    pub async fn reindex_users(
        self: &Arc<Self>,
        batch_size: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total = self.database.read_user_total().await?.try_into().unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.clone();

        let handle = task::spawn(async move {
            let index = &repo.search.indices.user;
            let stream = repo.database.read_user_search();

            repo.reindex(guard.task(), index, batch_size, stream).await
        });

        Ok((task, handle))
    }
}
