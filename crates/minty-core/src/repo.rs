pub mod optional_user;
pub mod with_user;

mod admin;
mod entity;
mod links;
mod object;
mod objects;
mod sessions;
mod task;
mod tasks;

pub use admin::Admin;
pub use object::Object;
pub use objects::Objects;
pub use optional_user::OptionalUser;
pub use sessions::*;
pub use tasks::Tasks;
pub use with_user::WithUser;

use entity::Entity;
use links::Links;

use crate::{
    auth::Auth,
    cache::{self, Cache, Cached},
    conf::RepoConfig,
    db::{self, Database, Password},
    error::{Found, Result},
    ico::Favicons,
    obj::Bucket,
    search::Search,
    task::Task,
    About, Error, SessionId,
};

use fstore::RemoveResult;
use minty::{export, Login, SignUp, Uuid};
use std::{path::Path, result, sync::Arc};

pub struct Repo {
    auth: Auth,
    bucket: Bucket,
    cache: Cache,
    database: Database,
    db_support: pgtools::Database,
    favicons: Favicons,
    require_account: bool,
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
        let cache = Cache::new(database.clone(), bucket.clone(), &config.cache);
        let favicons = Favicons::new(bucket.clone());

        Ok(Self {
            auth: Auth::new(),
            bucket,
            cache,
            database,
            db_support,
            favicons,
            require_account: config.require_account,
            search: Search::new(&config.search)?,
        })
    }

    fn about(&self) -> About {
        About {
            version: crate::VERSION,
        }
    }

    pub fn admin(&self, user: Arc<Cached<cache::User>>) -> Result<Admin> {
        Admin::new(self, user)
    }

    pub async fn authenticate(&self, login: &Login) -> Result<SessionId> {
        const ERROR: Option<&str> = Some("invalid credentials");

        let Some(Password { user_id, password }) =
            self.database.read_user_password(&login.email).await?
        else {
            return Err(Error::Unauthenticated(ERROR));
        };

        if !self.auth.verify_password(&login.password, &password)? {
            return Err(Error::Unauthenticated(ERROR));
        }

        let user = self
            .cache
            .users()
            .get(user_id)
            .await?
            .found("user", user_id)?;

        self.with_user(user).create_session().await
    }

    fn entity(&self, id: Uuid) -> Entity {
        Entity::new(self, id)
    }

    pub async fn grant_admin(&self, user: Uuid) -> Result<()> {
        self.database
            .update_admin(user, true)
            .await?
            .found("user", user)
    }

    fn links(&self) -> Links {
        Links::new(self)
    }

    pub fn object(&self, id: Uuid) -> Object {
        Object::new(self, id)
    }

    fn objects(&self) -> Objects {
        Objects::new(self)
    }

    pub fn optional_user(
        &self,
        user: Option<Arc<Cached<cache::User>>>,
    ) -> Result<OptionalUser> {
        if self.require_account && user.is_none() {
            Err(Error::Unauthenticated(Some("login required")))
        } else {
            Ok(OptionalUser::new(self, user))
        }
    }

    pub fn sessions(&self) -> Sessions {
        Sessions::new(self)
    }

    pub async fn sign_up(&self, info: SignUp) -> Result<SessionId> {
        let name = info.username.as_ref();
        let email = info.email.as_ref();
        let password = self.auth.hash_password(info.password)?;

        let mut tx = self.database.begin().await?;

        let user =
            tx.create_user(name, email, &password)
                .await
                .map_err(|err| {
                    err.as_database_error()
                        .and_then(|e| e.constraint())
                        .and_then(|constraint| match constraint {
                            "user_account_email_key" => {
                                Some(Error::AlreadyExists {
                                    entity: "user",
                                    identifier: format!(
                                        "email address '{email}'"
                                    ),
                                })
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| err.into())
                })?;

        self.search.add_user_alias(user.id, name).await?;

        tx.commit().await?;

        let user = self.cache.users().insert(user);
        self.with_user(user).create_session().await
    }

    fn task(self: &Arc<Self>, task: Task) -> task::Task {
        task::Task::new(self, task)
    }

    pub fn tasks<'a>(self: &'a Arc<Self>) -> Tasks<'a> {
        Tasks::new(self)
    }

    pub fn with_user(&self, user: Arc<Cached<cache::User>>) -> WithUser {
        WithUser::new(self, user)
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
