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
    pub user: Option<UserPreview>,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    #[sqlx(rename = "indent", try_from = "i16")]
    pub level: u16,
    pub content: String,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
}

impl From<Comment> for minty::Comment {
    fn from(value: Comment) -> Self {
        Self {
            id: value.id,
            post_id: value.post_id,
            parent_id: value.parent_id,
            level: value.level,
            content: value.content,
            created: value.created,
        }
    }
}

impl From<Comment> for minty::CommentData {
    fn from(value: Comment) -> Self {
        Self {
            id: value.id,
            content: value.content,
            level: value.level.try_into().unwrap(),
            created: value.created,
        }
    }
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
}

impl<'r> Decode<'r, Postgres> for Object {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut decoder = PgRecordDecoder::new(value)?;

        let id: Uuid = decoder.try_decode()?;
        let preview_id: Option<Uuid> = decoder.try_decode()?;

        Ok(Self { id, preview_id })
    }
}

impl Encode<'_, Postgres> for Object {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);

        encoder.encode(self.id);
        encoder.encode(self.preview_id);

        encoder.finish();
        IsNull::No
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
    pub poster: Option<UserPreview>,
    pub title: String,
    pub description: String,
    pub objects: Vec<Object>,
    pub posts: Vec<PostPreview>,
    pub tags: Vec<TagPreview>,
    #[sqlx(try_from = "i32")]
    pub comment_count: u32,
    pub visibility: Visibility,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
    #[sqlx(rename = "date_modified")]
    pub modified: DateTime,
}

#[derive(Clone, Debug, FromRow)]
pub struct PostPreview {
    #[sqlx(rename = "post_id")]
    pub id: Uuid,
    pub poster: Option<UserPreview>,
    pub title: String,
    pub preview: Option<Object>,
    #[sqlx(try_from = "i32")]
    pub comment_count: u32,
    #[sqlx(try_from = "i32")]
    pub object_count: u32,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
}

impl<'r> Decode<'r, Postgres> for PostPreview {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut decoder = PgRecordDecoder::new(value)?;

        let id: Uuid = decoder.try_decode()?;
        let poster: Option<UserPreview> = decoder.try_decode()?;
        let title: String = decoder.try_decode()?;
        let preview: Option<Object> = decoder.try_decode()?;
        let comment_count: i32 = decoder.try_decode()?;
        let object_count: i32 = decoder.try_decode()?;
        let created: DateTime = decoder.try_decode()?;

        Ok(Self {
            id,
            poster,
            title,
            preview,
            comment_count: comment_count.try_into()?,
            object_count: object_count.try_into()?,
            created,
        })
    }
}

impl Encode<'_, Postgres> for PostPreview {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);

        encoder.encode(self.id);
        encoder.encode(&self.title);
        encoder.encode(&self.preview);
        encoder.encode(self.comment_count as i32);
        encoder.encode(self.object_count as i32);
        encoder.encode(self.created);

        encoder.finish();
        IsNull::No
    }
}

impl Type<Postgres> for PostPreview {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("post_preview")
    }
}

impl PgHasArrayType for PostPreview {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_post_preview")
    }
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
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);

        encoder.encode(self.id);
        encoder.encode(&self.url);
        encoder.encode(self.icon);

        encoder.finish();
        IsNull::No
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
    pub creator: Option<UserPreview>,
    #[sqlx(try_from = "i32")]
    pub post_count: u32,
}

impl From<Tag> for minty::Tag {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id,
            profile: value.profile.into(),
            creator: value.creator.map(Into::into),
            post_count: value.post_count,
        }
    }
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

#[derive(Clone, Debug, FromRow)]
pub struct TagPreview {
    #[sqlx(rename = "tag_id")]
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<Uuid>,
}

impl<'r> Decode<'r, Postgres> for TagPreview {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut decoder = PgRecordDecoder::new(value)?;

        let id: Uuid = decoder.try_decode()?;
        let name: String = decoder.try_decode()?;
        let avatar: Option<Uuid> = decoder.try_decode()?;

        Ok(Self { id, name, avatar })
    }
}

impl Encode<'_, Postgres> for TagPreview {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);

        encoder.encode(self.id);
        encoder.encode(&self.name);
        encoder.encode(self.avatar);

        encoder.finish();
        IsNull::No
    }
}

impl Type<Postgres> for TagPreview {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("tag_preview")
    }
}

impl PgHasArrayType for TagPreview {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_tag_preview")
    }
}

impl From<TagPreview> for minty::TagPreview {
    fn from(value: TagPreview) -> Self {
        Self {
            id: value.id,
            name: value.name,
            avatar: value.avatar,
        }
    }
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
            profile: value.profile.into(),
            post_count: value.post_count,
            comment_count: value.comment_count,
            tag_count: value.tag_count,
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct UserPreview {
    #[sqlx(rename = "user_id")]
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<Uuid>,
}

impl<'r> Decode<'r, Postgres> for UserPreview {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut decoder = PgRecordDecoder::new(value)?;

        let id: Uuid = decoder.try_decode()?;
        let name: String = decoder.try_decode()?;
        let avatar: Option<Uuid> = decoder.try_decode()?;

        Ok(Self { id, name, avatar })
    }
}

impl Encode<'_, Postgres> for UserPreview {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut encoder = PgRecordEncoder::new(buf);

        encoder.encode(self.id);
        encoder.encode(&self.name);
        encoder.encode(self.avatar);

        encoder.finish();
        IsNull::No
    }
}

impl Type<Postgres> for UserPreview {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("user_preview")
    }
}

impl PgHasArrayType for UserPreview {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_user_preview")
    }
}

impl From<UserPreview> for minty::UserPreview {
    fn from(value: UserPreview) -> Self {
        Self {
            id: value.id,
            name: value.name,
            avatar: value.avatar,
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
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
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

    fn compatible(ty: &PgTypeInfo) -> bool {
        // Workaround for https://github.com/launchbadge/sqlx/issues/2831
        // sqlx::Type macro doesn't work with types in schemas outside search_path
        *ty == PgTypeInfo::with_name("visibility")
    }
}
