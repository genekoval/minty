#[cfg(feature = "export")]
pub mod export;

use crate::text;

pub use url::Url;
pub use uuid::Uuid;

use chrono::Local;
use std::{
    error::Error,
    fmt::{self, Display},
    str::FromStr,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type DateTime = chrono::DateTime<Local>;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct About {
    pub version: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Comment {
    pub id: Uuid,
    pub user: Option<UserPreview>,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub level: u8,
    pub content: String,
    pub created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CommentData {
    pub id: Uuid,
    pub user: Option<UserPreview>,
    pub content: String,
    pub level: u8,
    pub created: DateTime,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct EntityProfile {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub sources: Vec<Source>,
    pub avatar: Option<Uuid>,
    pub banner: Option<Uuid>,
    pub created: DateTime,
}

impl EntityProfile {
    pub fn add_source(&mut self, source: Source) {
        if let Err(index) =
            self.sources.binary_search_by(|s| s.url.cmp(&source.url))
        {
            self.sources.insert(index, source);
        }
    }

    pub fn delete_source(&mut self, id: i64) {
        if let Some(index) =
            self.sources.iter().position(|source| source.id == id)
        {
            self.sources.remove(index);
        }
    }

    pub fn delete_sources(&mut self, ids: &[i64]) {
        for id in ids.iter().copied() {
            self.delete_source(id);
        }
    }

    pub fn set_names(&mut self, names: &ProfileName) {
        self.name.clone_from(&names.name);
        self.aliases.clone_from(&names.aliases);
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Login {
    pub email: String,
    pub password: String,
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
    pub extension: Option<String>,
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

#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ObjectPreview {
    pub id: Uuid,
    pub preview_id: Option<Uuid>,
    pub r#type: String,
    pub subtype: String,
    pub extension: Option<String>,
}

impl PartialEq for ObjectPreview {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ObjectSummary {
    pub media_type: String,
    pub size: u64,
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
    pub poster: Option<UserPreview>,
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

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostParts {
    pub title: Option<text::PostTitle>,
    pub description: Option<text::Description>,
    pub visibility: Option<Visibility>,
    pub objects: Option<Vec<Uuid>>,
    pub posts: Option<Vec<Uuid>>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostPreview {
    pub id: Uuid,
    pub poster: Option<UserPreview>,
    pub title: String,
    pub preview: Option<ObjectPreview>,
    pub comment_count: u32,
    pub object_count: u32,
    pub created: DateTime,
}

impl PartialEq for PostPreview {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PostQuery {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub pagination: Pagination,

    #[cfg_attr(feature = "serde", serde(default))]
    pub poster: Option<Uuid>,

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

#[derive(Debug)]
pub enum ParsePostSortError {
    InvalidOrder(String),
    InvalidValue(String),
    TrailingText(String),
}

impl Display for ParsePostSortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOrder(order) => {
                write!(f, "invalid sort order '{order}'")
            }
            Self::InvalidValue(value) => {
                write!(f, "invalid post sort value '{value}'")
            }
            Self::TrailingText(text) => {
                write!(f, "unexpected trailing text '{text}'")
            }
        }
    }
}

impl Error for ParsePostSortError {}

impl FromStr for PostSort {
    type Err = ParsePostSortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PostSortValue::*;
        use SortOrder::*;

        let mut tokens = s.split('.');

        let value = match tokens.next().unwrap() {
            "created" => Created,
            "modified" => Modified,
            "relevance" => Relevance,
            "title" => Title,
            token => {
                return Err(ParsePostSortError::InvalidValue(token.into()))
            }
        };

        let order = match tokens.next() {
            Some(token) => {
                if token == "ascending" || token == "asc" {
                    Ascending
                } else if token == "descending" || token == "desc" {
                    Descending
                } else {
                    return Err(ParsePostSortError::InvalidOrder(token.into()));
                }
            }
            None => value.default_order(),
        };

        if let Some(token) = tokens.next() {
            return Err(ParsePostSortError::TrailingText(token.into()));
        }

        Ok(Self { value, order })
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
pub struct ProfileName {
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ProfileQuery {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub pagination: Pagination,

    pub name: String,

    #[cfg_attr(feature = "serde", serde(default))]
    pub exclude: Vec<Uuid>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SearchResult<T> {
    pub total: u32,
    pub hits: Vec<T>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SignUp {
    pub username: text::Name,
    pub email: text::Email,
    pub password: text::Password,
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

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.url.fmt(f)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Tag {
    pub id: Uuid,
    pub profile: EntityProfile,
    pub creator: Option<UserPreview>,
    pub post_count: u32,
}

#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TagPreview {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<Uuid>,
}

impl PartialEq for TagPreview {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub admin: bool,
    pub profile: EntityProfile,
    pub post_count: u32,
    pub comment_count: u32,
    pub tag_count: u32,
}

#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct UserPreview {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<Uuid>,
}

impl PartialEq for UserPreview {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Visibility {
    Draft,
    #[default]
    Public,
}

impl Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Visibility::*;

        let string = match self {
            Draft => "Draft",
            Public => "Public",
        };

        f.write_str(string)
    }
}
