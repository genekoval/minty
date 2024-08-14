pub use minty::{DateTime, Url, Uuid};

use serde::Serialize;
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef,
    },
    Decode, Encode, FromRow, Postgres, Type,
};

pub trait Id {
    fn id(&self) -> Uuid;
}

#[derive(Clone, Debug, FromRow)]
pub struct Comment {
    #[sqlx(rename = "comment_id")]
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    #[sqlx(rename = "indent", try_from = "i16")]
    pub level: u16,
    pub content: String,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
}

#[derive(Clone, Debug, FromRow)]
pub struct EntityProfile {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub sources: Vec<Source>,
    pub avatar: Option<Uuid>,
    pub banner: Option<Uuid>,
    pub created: DateTime,
}

impl From<EntityProfile> for minty::EntityProfile {
    fn from(value: EntityProfile) -> Self {
        Self {
            name: value.name,
            aliases: value.aliases,
            description: value.description,
            sources: value.sources.into_iter().map(Into::into).collect(),
            avatar: value.avatar,
            banner: value.banner,
            created: value.created,
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct Object {
    #[sqlx(rename = "object_id")]
    pub id: Uuid,
    pub preview_id: Option<Uuid>,
    pub posts: Vec<Uuid>,
}

impl<'r> Decode<'r, Postgres> for Object {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut decoder = PgRecordDecoder::new(value)?;

        Ok(Self {
            id: decoder.try_decode()?,
            preview_id: decoder.try_decode()?,
            posts: decoder.try_decode()?,
        })
    }
}

impl Encode<'_, Postgres> for Object {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        let mut encoder = PgRecordEncoder::new(buf);

        encoder.encode(self.id)?;
        encoder.encode(self.preview_id)?;

        encoder.finish();
        Ok(IsNull::No)
    }
}

impl Type<Postgres> for Object {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("object")
    }
}

impl PgHasArrayType for Object {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_object")
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct ObjectError {
    #[sqlx(rename = "object_id")]
    pub id: Uuid,
    pub message: String,
}

impl From<ObjectError> for minty::ObjectError {
    fn from(value: ObjectError) -> Self {
        Self {
            id: value.id,
            message: value.message,
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct Post {
    #[sqlx(rename = "post_id")]
    pub id: Uuid,
    pub poster: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub objects: Vec<Uuid>,
    pub posts: Vec<Uuid>,
    pub tags: Vec<Uuid>,
    #[sqlx(try_from = "i32")]
    pub comment_count: u32,
    pub visibility: Visibility,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
    #[sqlx(rename = "date_modified")]
    pub modified: DateTime,
}

impl Post {
    pub fn search(&self) -> PostSearch {
        PostSearch {
            id: self.id,
            poster: self.poster,
            title: self.title.clone(),
            description: self.description.clone(),
            visibility: self.visibility,
            created: self.created,
            modified: self.modified,
            tags: self.tags.clone(),
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct PostObjects {
    pub modified: DateTime,
    pub objects: Vec<Uuid>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct PostSearch {
    #[serde(skip)]
    #[sqlx(rename = "post_id")]
    pub id: Uuid,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<Uuid>,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,

    pub visibility: Visibility,

    #[sqlx(rename = "date_created")]
    pub created: DateTime,

    #[sqlx(rename = "date_modified")]
    pub modified: DateTime,

    pub tags: Vec<Uuid>,
}

impl Id for PostSearch {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct Session {
    pub user_id: Uuid,
    pub expiration: DateTime,
}

#[derive(Clone, Debug, FromRow, Type)]
#[sqlx(type_name = "site")]
pub struct Site {
    pub site_id: i64,
    pub scheme: String,
    pub host: String,
    pub icon: Option<Uuid>,
}

#[derive(Clone, Debug, FromRow)]
pub struct Source {
    #[sqlx(rename = "source_id")]
    pub id: i64,
    pub url: String,
    pub icon: Option<Uuid>,
}

impl<'r> Decode<'r, Postgres> for Source {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut decoder = PgRecordDecoder::new(value)?;

        let id: i64 = decoder.try_decode()?;
        let url: String = decoder.try_decode()?;
        let icon: Option<Uuid> = decoder.try_decode()?;

        Ok(Self { id, url, icon })
    }
}

impl Encode<'_, Postgres> for Source {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        let mut encoder = PgRecordEncoder::new(buf);

        encoder.encode(self.id)?;
        encoder.encode(&self.url)?;
        encoder.encode(self.icon)?;

        encoder.finish();
        Ok(IsNull::No)
    }
}

impl Type<Postgres> for Source {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("source")
    }
}

impl PgHasArrayType for Source {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_source")
    }
}

impl From<Source> for minty::Source {
    fn from(value: Source) -> Self {
        Self {
            id: value.id,
            url: Url::parse(&value.url).unwrap(),
            icon: value.icon,
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct Tag {
    #[sqlx(rename = "tag_id")]
    pub id: Uuid,
    #[sqlx(flatten)]
    pub profile: EntityProfile,
    pub creator: Option<Uuid>,
    #[sqlx(try_from = "i32")]
    pub post_count: u32,
}

#[derive(Clone, Debug, FromRow)]
pub struct Password {
    pub user_id: Uuid,
    pub password: String,
}

#[derive(Clone, Debug, FromRow, Type)]
#[sqlx(type_name = "profile_name")]
pub struct ProfileName {
    pub name: String,
    pub aliases: Vec<String>,
}

impl From<ProfileName> for minty::ProfileName {
    fn from(value: ProfileName) -> Self {
        Self {
            name: value.name,
            aliases: value.aliases,
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct ProfileNameUpdate {
    pub names: ProfileName,
    pub old_name: Option<String>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct TagSearch {
    #[serde(skip)]
    #[sqlx(rename = "tag_id")]
    pub id: Uuid,
    pub names: Vec<String>,
}

impl Id for TagSearch {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct User {
    #[sqlx(rename = "user_id")]
    pub id: Uuid,
    pub email: String,
    pub admin: bool,
    #[sqlx(flatten)]
    pub profile: EntityProfile,
    #[sqlx(try_from = "i32")]
    pub post_count: u32,
    #[sqlx(try_from = "i32")]
    pub comment_count: u32,
    #[sqlx(try_from = "i32")]
    pub tag_count: u32,
}

impl From<User> for minty::User {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            admin: value.admin,
            profile: value.profile.into(),
            post_count: value.post_count,
            comment_count: value.comment_count,
            tag_count: value.tag_count,
        }
    }
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct UserSearch {
    #[serde(skip)]
    #[sqlx(rename = "user_id")]
    pub id: Uuid,
    pub names: Vec<String>,
}

impl Id for UserSearch {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Draft,
    Public,
}

impl Visibility {
    pub fn from_minty(value: minty::Visibility) -> Self {
        use minty::Visibility::*;

        match value {
            Draft => Self::Draft,
            Public => Self::Public,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Public => "public",
        }
    }
}

impl From<Visibility> for minty::Visibility {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Draft => Self::Draft,
            Visibility::Public => Self::Public,
        }
    }
}

impl<'r> Decode<'r, Postgres> for Visibility {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = <&'r str as Decode<'r, Postgres>>::decode(value)?;

        match value {
            "draft" => Ok(Self::Draft),
            "public" => Ok(Self::Public),
            _ => {
                Err(format!("invalid value {value:?} for enum Visibility")
                    .into())
            }
        }
    }
}

impl Encode<'_, Postgres> for Visibility {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        let val = self.as_str();
        <&str as Encode<'_, Postgres>>::encode(val, buf)
    }

    fn size_hint(&self) -> usize {
        let val = self.as_str();
        <&str as Encode<'_, Postgres>>::size_hint(&val)
    }
}

impl Type<Postgres> for Visibility {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("data.visibility")
    }
}
