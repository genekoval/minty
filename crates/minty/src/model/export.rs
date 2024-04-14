use super::{DateTime, Deserialize, Serialize, Url, Uuid, Visibility};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub posts: Vec<Post>,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Comment {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub indent: u8,
    pub content: String,
    pub created: DateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub visibility: Visibility,
    pub created: DateTime,
    pub modified: DateTime,
    pub objects: Vec<Uuid>,
    pub posts: Vec<Uuid>,
    pub tags: Vec<Uuid>,
    pub comments: Vec<Comment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Source {
    pub url: Url,
    pub icon: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub avatar: Option<Uuid>,
    pub banner: Option<Uuid>,
    pub sources: Vec<Source>,
    pub created: DateTime,
}
