use super::{DateTime, Deserialize, Serialize, Url, Uuid, Visibility};

pub trait Profile {
    fn id(&self) -> Uuid;

    fn profile(&self) -> &EntityProfile;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub posts: Vec<Post>,
    pub tags: Vec<Tag>,
    pub users: Vec<User>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Comment {
    pub id: Uuid,
    pub user: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub indent: u8,
    pub content: String,
    pub created: DateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntityProfile {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub sources: Vec<Source>,
    pub avatar: Option<Uuid>,
    pub banner: Option<Uuid>,
    pub created: DateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: Uuid,
    pub poster: Option<Uuid>,
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
    #[serde(flatten)]
    pub profile: EntityProfile,
    pub creator: Option<Uuid>,
}

impl Profile for Tag {
    fn id(&self) -> Uuid {
        self.id
    }

    fn profile(&self) -> &EntityProfile {
        &self.profile
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    #[serde(flatten)]
    pub profile: EntityProfile,
}

impl Profile for User {
    fn id(&self) -> Uuid {
        self.id
    }

    fn profile(&self) -> &EntityProfile {
        &self.profile
    }
}
