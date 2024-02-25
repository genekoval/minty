pub use minty::{DateTime, Url, Uuid};

use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    postgres::{
        types::{PgRecordDecoder, PgRecordEncoder},
        PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef,
    },
    Decode, Encode, FromRow, Postgres, Type,
};

#[derive(Clone, Debug, FromRow)]
pub struct Comment {
    #[sqlx(rename = "comment_id")]
    pub id: Uuid,
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

#[derive(Clone, Debug, FromRow)]
pub struct Post {
    #[sqlx(rename = "post_id")]
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub objects: Vec<Object>,
    pub comment_count: i32,
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
    pub title: String,
    pub preview: Option<Object>,
    pub comment_count: i32,
    pub object_count: i32,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
}

#[derive(Clone, Debug, FromRow)]
pub struct PostSearch {
    #[sqlx(rename = "post_id")]
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub visibility: Visibility,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
    #[sqlx(rename = "date_modified")]
    pub modified: DateTime,
    pub tags: Vec<Uuid>,
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
    pub resource: String,
    pub site: Site,
}

impl From<Source> for minty::Source {
    fn from(value: Source) -> Self {
        let scheme = value.site.scheme;
        let host = value.site.host;
        let resource = value.resource;

        let url = format!("{scheme}://{host}{resource}");
        let url = Url::parse(&url).unwrap();

        Self {
            id: value.id,
            url,
            icon: value.site.icon,
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct Tag {
    #[sqlx(rename = "tag_id")]
    pub id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub avatar: Option<Uuid>,
    pub banner: Option<Uuid>,
    #[sqlx(try_from = "i32")]
    pub post_count: u32,
    #[sqlx(rename = "date_created")]
    pub created: DateTime,
}

impl From<Tag> for minty::Tag {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id,
            name: value.name,
            aliases: value.aliases,
            description: value.description,
            avatar: value.avatar,
            banner: value.banner,
            sources: vec![],
            post_count: value.post_count,
            created: value.created,
        }
    }
}

#[derive(Clone, Debug, FromRow, Type)]
#[sqlx(type_name = "tag_name")]
pub struct TagName {
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, FromRow)]
pub struct TagNameUpdate {
    pub names: TagName,
    pub old_name: Option<String>,
}

#[derive(Clone, Debug, FromRow)]
pub struct TagPreview {
    #[sqlx(rename = "tag_id")]
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<Uuid>,
}

#[derive(Clone, Debug, FromRow)]
pub struct TagSearch {
    #[sqlx(rename = "tag_id")]
    pub id: Uuid,
    pub names: Vec<String>,
}

#[derive(Clone, Copy, Debug, Type)]
#[sqlx(type_name = "visibility", rename_all = "lowercase")]
pub enum Visibility {
    Draft,
    Public,
}

impl From<Visibility> for minty::Visibility {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Draft => Self::Draft,
            Visibility::Public => Self::Public,
        }
    }
}
