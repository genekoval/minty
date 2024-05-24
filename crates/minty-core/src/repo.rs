mod comment;
mod entity;
mod links;
mod object;
mod objects;
mod post;
mod posts;
mod tag;
mod tags;
mod task;
mod tasks;
mod user;
mod users;

pub use comment::Comment;
pub use object::Object;
pub use objects::Objects;
pub use post::Post;
pub use posts::Posts;
pub use tag::Tag;
pub use tags::Tags;
pub use tasks::Tasks;
pub use user::User;
pub use users::Users;

use entity::Entity;
use links::Links;

use crate::{
    auth::Auth,
    cache::Cache,
    conf::RepoConfig,
    db::{self, Database},
    error::Result,
    ico::Favicons,
    obj::Bucket,
    search::Search,
    task::Task,
    About,
};

use fstore::RemoveResult;
use minty::{export, Uuid};
use std::{path::Path, result, sync::Arc};

pub struct Repo {
    auth: Auth,
    bucket: Bucket,
    cache: Cache,
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

        let bucket = Bucket::new(&config.objects).await?;
        let favicons = Favicons::new(bucket.clone());

        Ok(Self {
            auth: Auth::new(),
            bucket,
            cache: Cache::new(&config.cache),
            database: Database::new(pool),
            db_support,
            favicons,
            search: Search::new(&config.search)?,
        })
    }

    pub fn about(&self) -> About {
        About {
            version: crate::VERSION,
        }
    }

    pub fn comment(&self, id: Uuid) -> Comment {
        Comment::new(self, id)
    }

    fn entity(&self, id: Uuid) -> Entity {
        Entity::new(self, id)
    }

    fn links(&self) -> Links {
        Links::new(self)
    }

    pub fn object(&self, id: Uuid) -> Object {
        Object::new(self, id)
    }

    pub fn objects(&self) -> Objects {
        Objects::new(self)
    }

    pub fn post(&self, id: Uuid) -> Post {
        Post::new(self, id)
    }

    pub fn posts(&self) -> Posts {
        Posts::new(self)
    }

    pub fn tag(&self, id: Uuid) -> Tag {
        Tag::new(self, id)
    }

    pub fn tags(&self) -> Tags {
        Tags::new(self)
    }

    fn task(self: &Arc<Self>, task: Task) -> task::Task {
        task::Task::new(self, task)
    }

    pub fn tasks<'a>(self: &'a Arc<Self>) -> Tasks<'a> {
        Tasks::new(self)
    }

    pub fn user(&self, id: Uuid) -> User {
        User::new(self, id)
    }

    pub fn users(&self) -> Users {
        Users::new(self)
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
            self.objects().add(object).await?;
        }
        self.database.import(sqlx::types::Json(data)).await?;

        self.import_profile(&data.tags).await?;
        self.import_profile(&data.users).await?;

        Ok(())
    }

    async fn import_profile<P>(&self, entities: &[P]) -> Result<()>
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
}
