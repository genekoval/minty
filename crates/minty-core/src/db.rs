mod model;

pub use model::*;

pub use sqlx::postgres::PgPoolOptions as PoolOptions;

use sqlx_helper_macros::{database, transaction};

database! {
    create_comment(post_id: Uuid, content: &str) -> Comment;

    create_object(
        object_id: Uuid,
        preview_id: Option<Uuid>,
        source_id: Option<i64>,
    );

    create_object_preview_error(object_id: Uuid, message: &str);

    create_related_post(post_id: Uuid, related: Uuid);

    create_reply(parent_id: Uuid, content: &str) -> Comment;

    create_site(scheme: &str, name: &str, icon: Option<Uuid>) -> Site;

    create_source(site_id: i64, resource: &str) -> Source;

    create_tag_source(tag_id: Uuid, source_id: i64);

    delete_comment(id: Uuid, recursive: bool) -> bool;

    delete_object_preview_error(object_id: Uuid);

    delete_related_post(post_id: Uuid, related: Uuid);

    delete_tag_source(tag_id: Uuid, source_id: i64);

    prune();

    prune_objects() -> Vec<(Uuid,)>;

    read_comment(id: Uuid) -> Option<Comment>;

    read_comments(post_id: Uuid) -> Vec<Comment>;

    read_object(object_id: Uuid) -> Option<Object>;

    read_object_posts(object_id: Uuid) -> Vec<PostPreview>;

    read_object_preview_errors() -> Vec<ObjectError>;

    read_post(id: Uuid) -> Option<Post>;

    read_posts(posts: &[Uuid]) -> Vec<PostPreview>;

    read_post_search() -> Stream<PostSearch>;

    read_site(scheme: &str, host: &str) -> Option<(i64,)>;

    read_tag(id: Uuid) -> Option<Tag>;

    read_tag_previews(tags: &[Uuid]) -> Vec<TagPreview>;

    read_tag_sources(tag_id: Uuid) -> Vec<Source>;

    read_tag_search() -> Stream<TagSearch>;

    read_total_objects() -> i64;

    update_comment(comment_id: Uuid, content: &str);

    update_object_preview(object_id: Uuid, preview_id: Option<Uuid>);

    update_tag_description(tag_id: Uuid, description: &str) -> bool;

    stream_objects() -> Stream<Object>;
}

transaction! {
    create_post_objects(
        post_id: Uuid,
        objects: &[Uuid],
        destination: Option<Uuid>,
    ) -> (DateTime,);

    create_post(post_id: Uuid) -> (DateTime,);

    create_post_draft() -> PostSearch;

    create_post_tag(post_id: Uuid, tag_id: Uuid);

    create_tag(name: &str) -> (Uuid,);

    create_tag_alias(tag_id: Uuid, alias: &str) -> TagName;

    delete_post(id: Uuid);

    delete_post_objects(post_id: Uuid, objects: &[Uuid]) -> (DateTime,);

    delete_post_tag(post_id: Uuid, tag_id: Uuid);

    delete_tag(id: Uuid);

    delete_tag_alias(tag_id: Uuid, alias: &str) -> TagName;

    update_tag_name(tag_id: Uuid, name: &str) -> TagNameUpdate;

    update_post_description(
        post_id: Uuid,
        description: &str,
    ) -> Option<(DateTime,)>;

    update_post_title(post_id: Uuid, title: &str) -> Option<(DateTime,)>;
}
