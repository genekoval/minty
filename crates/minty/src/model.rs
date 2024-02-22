pub use url::Url;
pub use uuid::Uuid;

use chrono::Local;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type DateTime = chrono::DateTime<Local>;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct About {
    pub version: Version,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub level: u16,
    pub content: String,
    pub created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CommentData {
    pub id: Uuid,
    pub content: String,
    pub level: u8,
    pub created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Modification<T> {
    pub date_modified: DateTime,
    pub new_value: T,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Object {
    pub id: Uuid,
    pub hash: String,
    pub size: u64,
    pub r#type: String,
    pub subtype: String,
    pub added: DateTime,
    pub preview_id: Option<Uuid>,
    pub posts: Vec<PostPreview>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ObjectError {
    pub id: Uuid,
    pub message: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ObjectPreview {
    pub id: Uuid,
    pub preview_id: Option<Uuid>,
    pub r#type: String,
    pub subtype: String,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Pagination {
    #[cfg_attr(feature = "serde", serde(default))]
    pub from: u32,

    #[cfg_attr(feature = "serde", serde(default = "Pagination::default_size"))]
    pub size: u32,
}

impl Pagination {
    pub fn default_size() -> u32 {
        10
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            from: 0,
            size: Self::default_size(),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub visibility: Visibility,
    pub created: DateTime,
    pub modified: DateTime,
    pub objects: Vec<ObjectPreview>,
    pub posts: Vec<PostPreview>,
    pub tags: Vec<TagPreview>,
    pub comment_count: u32,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostPreview {
    pub id: Uuid,
    pub title: String,
    pub preview: Option<ObjectPreview>,
    pub comment_count: u32,
    pub object_count: u32,
    pub created: DateTime,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostQuery {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub pagination: Pagination,

    #[cfg_attr(feature = "serde", serde(default))]
    pub text: String,

    #[cfg_attr(feature = "serde", serde(default))]
    pub tags: Vec<Uuid>,

    #[cfg_attr(feature = "serde", serde(default, alias = "vis"))]
    pub visibility: Visibility,

    #[cfg_attr(feature = "serde", serde(default))]
    pub sort: PostSort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostSort {
    pub value: PostSortValue,
    pub order: SortOrder,
}

impl PostSort {
    pub const CREATED: Self = Self {
        value: PostSortValue::Created,
        order: PostSortValue::Created.default_order(),
    };

    pub const MODIFIED: Self = Self {
        value: PostSortValue::Modified,
        order: PostSortValue::Modified.default_order(),
    };

    pub const RELEVANCE: Self = Self {
        value: PostSortValue::Relevance,
        order: PostSortValue::Relevance.default_order(),
    };

    pub const TITLE: Self = Self {
        value: PostSortValue::Title,
        order: PostSortValue::Title.default_order(),
    };
}

impl Default for PostSort {
    fn default() -> Self {
        let value = PostSortValue::default();
        let order = value.default_order();

        Self { value, order }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum PostSortValue {
    #[default]
    Created,
    Modified,
    Relevance,
    Title,
}

impl PostSortValue {
    pub const fn default_order(&self) -> SortOrder {
        use SortOrder::*;

        match self {
            Self::Title => Ascending,
            _ => Descending,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SearchResult<T> {
    pub total: u32,
    pub hits: Vec<T>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SortOrder {
    #[cfg_attr(feature = "serde", serde(rename = "asc"))]
    Ascending,
    #[cfg_attr(feature = "serde", serde(rename = "desc"))]
    Descending,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Source {
    pub id: i64,
    pub url: Url,
    pub icon: Option<Uuid>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub avatar: Option<Uuid>,
    pub banner: Option<Uuid>,
    pub sources: Vec<Source>,
    pub post_count: u32,
    pub created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TagName {
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TagPreview {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<Uuid>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TagQuery {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub pagination: Pagination,

    pub name: String,

    #[cfg_attr(feature = "serde", serde(default))]
    pub exclude: Vec<Uuid>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Version {
    pub number: String,
    pub branch: String,
    pub build_time: String,
    pub build_os: String,
    pub build_type: String,
    pub commit_hash: String,
    pub commit_date: String,
    pub rust_version: String,
    pub rust_channel: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Visibility {
    Draft,
    #[default]
    Public,
}
