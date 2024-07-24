#![allow(clippy::too_many_arguments)]

mod model;

pub use model::*;

use crate::conf::DatabaseConfig;

use core::time::Duration;
use log::LevelFilter;
use minty::model::export::Data;
use sqlx::{
    postgres::{
        PgConnectOptions as ConnectOptions, PgPoolOptions as PoolOptions,
    },
    types::Json,
    ConnectOptions as _,
};
use sqlx_helper_macros::{database, transaction};

database! {
    create_comment(user_id: Uuid, post_id: Uuid, content: &str) -> Comment;

    create_entity_link(profile_id: Uuid, source_id: i64);

    create_object(
        object_id: Uuid,
        preview_id: Option<Uuid>,
        source_id: Option<i64>,
    );

    create_object_preview_error(object_id: Uuid, message: &str);

    create_related_post(post_id: Uuid, related: Uuid) -> (Vec<Uuid>,);

    create_reply(
        user_id: Uuid,
        parent_id: Uuid,
        content: &str
    ) -> Option<Comment>;

    create_site(scheme: &str, name: &str, icon: Option<Uuid>) -> Site;

    create_source(site_id: i64, resource: &str) -> Source;

    create_user_session(user_id: Uuid, session_id: &[u8], expiration: DateTime);

    delete_comment(id: Uuid, recursive: bool) -> bool;

    delete_entity_link(profile_id: Uuid, source_id: i64) -> bool;

    delete_related_post(post_id: Uuid, related: Uuid) -> (Option<Vec<Uuid>>,);

    delete_user_session(session_id: &[u8]);

    export() -> (Json<Data>,);

    import(data: Json<&Data>);

    prune();

    read_comment_post(id: Uuid) -> (Option<Uuid>,);

    read_comments(post_id: Uuid) -> Vec<Comment>;

    read_entity_sources(profile_id: Uuid) -> Vec<Source>;

    read_object(object_id: Uuid) -> Option<Object>;

    read_object_preview_errors() -> Vec<ObjectError>;

    read_object_total() -> i64;

    read_objects(objects: &[Uuid]) -> Vec<Object>;

    read_post(id: Uuid) -> Option<Post>;

    read_posts(posts: &[Uuid]) -> Vec<Post>;

    read_post_search() -> Stream<PostSearch>;

    read_post_total() -> i64;

    read_site(scheme: &str, host: &str) -> (Option<i64>,);

    read_tag(id: Uuid) -> Option<Tag>;

    read_tag_search() -> Stream<TagSearch>;

    read_tag_total() -> i64;

    read_tags(tags: &[Uuid]) -> Vec<Tag>;

    read_user(id: Uuid) -> Option<User>;

    read_user_password(email: &str) -> Option<Password>;

    read_user_search() -> Stream<UserSearch>;

    read_user_session(session_id: &[u8]) -> Option<Session>;

    read_user_total() -> i64;

    read_users(users: &[Uuid]) -> Vec<User>;

    stream_objects() -> Stream<Object>;

    update_admin(user_id: Uuid, admin: bool) -> bool;

    update_comment(comment_id: Uuid, content: &str) -> bool;

    update_entity_description(profile_id: Uuid, description: &str) -> bool;

    update_object_preview(object_id: Uuid, preview_id: Option<Uuid>);

    update_user_email(user_id: Uuid, email: &str) -> bool;

    update_user_password(user_id: Uuid, password: &str) -> bool;
}

transaction! {
    create_entity_alias(profile_id: Uuid, alias: &str) -> Option<ProfileName>;

    create_post(
        poster: Uuid,
        title: &str,
        description: &str,
        visibility: Option<Visibility>,
        objects: &[Uuid],
        posts: &[Uuid],
        tags: &[Uuid]
    ) -> Post;

    create_post_objects(
        post_id: Uuid,
        objects: &[Uuid],
        destination: Option<Uuid>,
    ) -> PostObjects;

    create_post_tag(post_id: Uuid, tag_id: Uuid);

    create_tag(name: &str, creator: Uuid) -> Tag;

    create_user(name: &str, email: &str, passowrd: &str) -> User;

    delete_entity(id: Uuid) -> bool;

    delete_entity_alias(profile_id: Uuid, alias: &str) -> Option<ProfileName>;

    delete_post(id: Uuid) -> bool;

    delete_post_objects(post_id: Uuid, objects: &[Uuid]) -> (DateTime,);

    delete_post_tag(post_id: Uuid, tag_id: Uuid) -> bool;

    prune_objects() -> Vec<(Uuid,)>;

    publish_post(post_id: Uuid) -> (DateTime,);

    update_entity_name(profile_id: Uuid, name: &str) -> Option<ProfileNameUpdate>;

    update_post_description(
        post_id: Uuid,
        description: &str,
    ) -> Option<(DateTime,)>;

    update_post_title(post_id: Uuid, title: &str) -> Option<(DateTime,)>;
}

impl Database {
    pub async fn from_config(config: &DatabaseConfig) -> Result<Self, String> {
        let url = config.connection.as_url();

        let options = ConnectOptions::from_url(&url)
            .map_err(|err| {
                format!("failed to create database connect options: {err}")
            })?
            .log_slow_statements(LevelFilter::Debug, Duration::from_secs(30));

        let pool = PoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await
            .map_err(|err| {
                format!("failed to establish database connection: {err}")
            })?;

        Ok(Self::new(pool))
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self::new(self.pool.clone())
    }
}
