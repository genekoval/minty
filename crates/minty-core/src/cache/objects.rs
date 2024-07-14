use super::{Cache, Cached, Id, Result, User};

use crate::db;

use minty::{DateTime, ObjectPreview, Uuid};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Object {
    pub id: Uuid,
    pub hash: String,
    pub size: u64,
    pub r#type: String,
    pub subtype: String,
    pub added: DateTime,
    pub preview_id: Option<Uuid>,
    posts: RwLock<Vec<Uuid>>,
}

impl Object {
    fn new(data: db::Object, obj: fstore::Object) -> Self {
        Self {
            id: data.id,
            hash: obj.hash,
            size: obj.size,
            r#type: obj.r#type,
            subtype: obj.subtype,
            added: obj.added,
            preview_id: data.preview_id,
            posts: RwLock::new(data.posts),
        }
    }

    pub async fn model(
        &self,
        cache: &Cache,
        user: Option<&Arc<Cached<User>>>,
    ) -> Result<minty::Object> {
        let posts = self.posts.read().unwrap().clone();
        let posts = cache.posts().previews(&posts, user).await?;

        Ok(minty::Object {
            id: self.id,
            hash: self.hash.clone(),
            size: self.size,
            r#type: self.r#type.clone(),
            subtype: self.subtype.clone(),
            added: self.added,
            preview_id: self.preview_id,
            posts,
        })
    }

    pub fn preview(&self) -> ObjectPreview {
        ObjectPreview {
            id: self.id,
            preview_id: self.preview_id,
            r#type: self.r#type.clone(),
            subtype: self.subtype.clone(),
        }
    }

    pub fn add_post(&self, id: Uuid) {
        let mut posts = self.posts.write().unwrap();

        if !posts.contains(&id) {
            posts.insert(0, id);
        }
    }

    pub fn delete_post(&self, id: Uuid) {
        self.posts.write().unwrap().retain(|post| *post != id);
    }
}

impl Id for Object {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

pub struct Objects<'a> {
    cache: &'a Cache,
}

impl<'a> Objects<'a> {
    pub(super) fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Arc<Cached<Object>>>> {
        self.cache
            .objects
            .get(id, || async {
                if let Some(object) =
                    self.cache.database.read_object(id).await?
                {
                    let metadata = self.cache.bucket.get_object(id).await?;
                    Ok(Some(Object::new(object, metadata)))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    pub async fn get_multiple(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Arc<Cached<Object>>>> {
        self.cache
            .objects
            .get_multiple(ids, |ids| async move {
                let objects = self.cache.database.read_objects(&ids).await?;
                let metadata = self.cache.bucket.get_objects(&ids).await?;

                Ok(objects
                    .into_iter()
                    .zip(metadata.into_iter())
                    .map(|(data, obj)| Object::new(data, obj))
                    .collect())
            })
            .await
    }
}
