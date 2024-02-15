pub use url::Url;
pub use uuid::Uuid;

use chrono::Local;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type DateTime = chrono::DateTime<Local>;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Comment {
    id: Uuid,
    post_id: Uuid,
    parent_id: Option<Uuid>,
    level: u8,
    content: String,
    created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CommentData {
    id: Uuid,
    content: String,
    level: u8,
    created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Modification<T> {
    date_modified: DateTime,
    new_value: T,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Object {
    id: Uuid,
    hash: String,
    size: u64,
    r#type: String,
    subtype: String,
    added: DateTime,
    preview_id: Option<Uuid>,
    posts: Vec<PostPreview>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ObjectError {
    id: Uuid,
    message: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ObjectPreview {
    id: Uuid,
    preview_id: Option<Uuid>,
    r#type: String,
    subtype: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Post {
    id: Uuid,
    title: String,
    description: String,
    visibility: Visibility,
    created: DateTime,
    modified: DateTime,
    objects: Vec<ObjectPreview>,
    posts: Vec<PostPreview>,
    tags: Vec<TagPreview>,
    comment_count: u32,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostPreview {
    id: Uuid,
    title: String,
    preview: Option<ObjectPreview>,
    comment_count: u32,
    object_count: u32,
    created: DateTime,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostQuery {
    from: u32,
    size: u32,
    text: Option<String>,
    tags: Vec<Uuid>,
    visibility: Visibility,
    sort: PostSort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostSort {
    value: PostSortValue,
    order: SortOrder,
}

impl PostSort {
    pub const CREATED: Self = Self {
        value: PostSortValue::Created,
        order: SortOrder::Descending,
    };

    pub const MODIFIED: Self = Self {
        value: PostSortValue::Modified,
        order: SortOrder::Descending,
    };

    pub const RELEVANCE: Self = Self {
        value: PostSortValue::Relevance,
        order: SortOrder::Descending,
    };

    pub const TITLE: Self = Self {
        value: PostSortValue::Title,
        order: SortOrder::Ascending,
    };
}

impl Default for PostSort {
    fn default() -> Self {
        Self::CREATED
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PostSortValue {
    Created,
    Modified,
    Relevance,
    Title,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SearchResult<T> {
    total: u32,
    hits: Vec<T>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Source {
    id: i64,
    url: Url,
    icon: Option<Uuid>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Tag {
    id: Uuid,
    name: String,
    aliases: Vec<String>,
    description: String,
    avatar: Option<Uuid>,
    banner: Option<Uuid>,
    sources: Vec<Source>,
    post_count: u32,
    created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TagName {
    name: String,
    aliases: Vec<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TagPreview {
    id: Uuid,
    name: String,
    avatar: Option<Uuid>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TagQuery {
    from: u32,
    size: u32,
    name: String,
    exclude: Vec<Uuid>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Visibility {
    Draft,
    #[default]
    Public,
}
