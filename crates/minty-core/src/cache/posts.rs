use super::{Cache, CacheLock, Cached, Comment, Id, Object, Result, Tag, User};

use crate::{db, error::Found, Error};

use dashmap::DashMap;
use minty::{CommentData, DateTime, PostPreview, Uuid, Visibility};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use tokio::task;

const EXISTING_COMMENT: &str =
    "comment path should always lead to an existing comment";

#[derive(Debug)]
pub struct PostMut {
    pub title: String,
    pub description: String,
    pub visibility: Visibility,
    pub created: DateTime,
    pub modified: DateTime,
    pub objects: Vec<Arc<Cached<Object>>>,
    pub posts: Vec<Uuid>,
    pub tags: Vec<Arc<Cached<Tag>>>,
    pub comment_count: u32,
    pub comments: Option<Vec<Comment>>,
    comment_map: Weak<CommentMap>,
}

impl PostMut {
    fn add_comment(&mut self, comment: Comment) -> Option<Box<[usize]>> {
        self.comment_count += 1;

        self.comments.as_mut().map(|comments| {
            comments.push(comment);
            vec![comments.len() - 1].into_boxed_slice()
        })
    }

    fn add_tag(&mut self, tag: Arc<Cached<Tag>>) {
        self.tags.retain(|tag| !tag.is_deleted());

        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    fn comment<F, R>(&mut self, path: &[usize], f: F) -> Option<R>
    where
        F: FnOnce(&mut Comment) -> R,
    {
        let comments = self.comments.as_mut()?;

        let first = *path.first().expect("empty paths should not exist");
        let mut comment = comments.get_mut(first).expect(EXISTING_COMMENT);

        for index in path[1..].iter().copied() {
            comment = comment.children.get_mut(index).expect(EXISTING_COMMENT);
        }

        Some(f(comment))
    }

    fn comments<F, R>(&self, id: Uuid, f: &F) -> Option<R>
    where
        F: Fn(PostComments<'_>) -> R,
    {
        let comments = PostComments {
            post: id,
            comments: self.comments.as_ref()?,
        };

        Some(f(comments))
    }

    fn delete_comment(
        &mut self,
        path: &[usize],
        recursive: bool,
    ) -> Vec<Comment> {
        let mut count = 0;
        let mut result = self
            .comment(path, |comment| {
                count = if recursive { comment.count() as u32 } else { 1 };
                comment.delete(recursive)
            })
            .unwrap_or_default();

        self.comment_count -= count;

        let mut pruned = false;

        for len in (1..path.len()).rev() {
            pruned = self
                .comment(&path[0..len], |comment| {
                    let prune =
                        comment.children.iter().all(|child| child.is_empty());

                    if prune {
                        result.append(&mut comment.children);
                    }

                    prune
                })
                .unwrap_or(pruned);

            if !pruned {
                break;
            }
        }

        if pruned {
            if let Some(comments) = self.comments.as_mut() {
                if comments.iter().all(|child| child.is_empty()) {
                    result.append(comments);
                }
            }
        }

        result
    }

    fn delete_tag(&mut self, id: Uuid) {
        self.tags.retain(|tag| !tag.is_deleted() && tag.id != id);
    }

    fn reply(&mut self, path: &[usize], reply: Comment) -> Option<usize> {
        self.comment(path, |parent| parent.reply(reply))
            .inspect(|_| self.comment_count += 1)
    }
}

impl Drop for PostMut {
    fn drop(&mut self) {
        if let Some(comments) = self.comments.take() {
            if !comments.is_empty() {
                if let Some(map) = self.comment_map.upgrade() {
                    map.delete(comments);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Post {
    pub id: Uuid,
    pub poster: Option<Arc<Cached<User>>>,
    mutable: CacheLock<PostMut>,
}

impl Post {
    async fn new(post: db::Post, cache: &Cache, is_new: bool) -> Result<Self> {
        let poster = if let Some(poster) = post.poster {
            cache.users().get(poster).await.ok().flatten()
        } else {
            None
        };

        let visibility = post.visibility.into();
        let objects = cache.objects().get_multiple(&post.objects).await?;
        let tags = cache.tags().get_multiple(&post.tags).await?;

        if is_new {
            objects.iter().for_each(|object| object.add_post(post.id));

            if visibility != Visibility::Draft {
                if let Some(user) = &poster {
                    user.update(|user| user.post_count += 1)
                }

                for tag in &tags {
                    tag.update(|tag| tag.post_count += 1);
                }
            }
        }

        Ok(Self {
            id: post.id,
            poster,
            mutable: CacheLock::new(PostMut {
                title: post.title,
                description: post.description,
                visibility,
                created: post.created,
                modified: post.modified,
                objects,
                posts: post.posts,
                tags,
                comment_count: post.comment_count,
                comments: None,
                comment_map: Arc::downgrade(&cache.comments),
            }),
        })
    }

    pub fn can_edit(&self, user: &Arc<Cached<User>>) -> Result<()> {
        let poster = self.poster.as_ref().map(|user| user.id);

        if poster == Some(user.id) {
            Ok(())
        } else {
            user.deny_permission()
        }
    }

    pub fn can_view(&self, user: Option<&Arc<Cached<User>>>) -> Result<()> {
        let is_draft = self
            .mutable
            .map(|post| post.visibility == Visibility::Draft)
            .found("post", self.id)?;

        let poster = self.poster.as_ref();
        let is_poster = user.is_some_and(|user| Some(user) == poster);

        if !is_draft || is_poster {
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    }

    pub async fn model(
        &self,
        cache: &Cache,
        user: Option<&Arc<Cached<User>>>,
    ) -> Result<Option<minty::Post>> {
        let Some(posts) = self.mutable.map(|post| post.posts.clone()) else {
            return Ok(None);
        };

        let posts = cache.posts().previews(&posts, user).await?;

        Ok(self.mutable.map(|post| minty::Post {
            id: self.id,
            poster: self.poster.as_ref().and_then(|user| user.preview()),
            title: post.title.clone(),
            description: post.description.clone(),
            visibility: post.visibility,
            created: post.created,
            modified: post.modified,
            objects: post
                .objects
                .iter()
                .map(|object| object.preview())
                .collect(),
            posts,
            tags: post.tags.iter().filter_map(|tag| tag.preview()).collect(),
            comment_count: post.comment_count,
        }))
    }

    pub fn preview(
        &self,
        user: Option<&Arc<Cached<User>>>,
    ) -> Option<minty::PostPreview> {
        self.mutable.and_then(|post| {
            let poster = self.poster.as_ref();
            let is_poster = user.is_some_and(|user| Some(user) == poster);

            (post.visibility != Visibility::Draft || is_poster).then(|| {
                PostPreview {
                    id: self.id,
                    poster: poster.and_then(|user| user.preview()),
                    title: post.title.clone(),
                    preview: post
                        .objects
                        .first()
                        .map(|object| object.preview()),
                    comment_count: post.comment_count,
                    object_count: post.objects.len().try_into().unwrap(),
                    created: post.created,
                }
            })
        })
    }

    pub fn add_comment(
        &self,
        this: &Arc<Cached<Post>>,
        cache: &Cache,
        comment: db::Comment,
        user: Arc<Cached<User>>,
    ) {
        user.update(|user| user.comment_count += 1);

        let id = comment.id;
        let comment = Comment::new(comment, Some(user));

        if let Some(path) = self
            .mutable
            .update(|post| post.add_comment(comment))
            .flatten()
        {
            cache.comments.insert(id, this, path);
        }
    }

    pub fn add_objects(
        &self,
        objects: Vec<Arc<Cached<Object>>>,
        modified: DateTime,
    ) {
        for object in &objects {
            object.add_post(self.id);
        }

        self.mutable.update(|post| {
            post.objects = objects;
            post.modified = modified
        });
    }

    async fn build_comments(
        &self,
        this: &Arc<Cached<Self>>,
        cache: &Cache,
    ) -> Result<Vec<Comment>> {
        let comments = cache.database.read_comments(self.id).await?;

        let mut users: HashMap<Uuid, Option<Arc<Cached<User>>>> = comments
            .iter()
            .filter_map(|comment| comment.user_id)
            .map(|id| (id, None))
            .collect();

        cache
            .users()
            .get_multiple(&users.keys().copied().collect::<Vec<_>>())
            .await?
            .into_iter()
            .for_each(|user| {
                users.insert(user.id, Some(user));
            });

        let mut result: Vec<Comment> = Vec::new();

        let levels: Vec<_> =
            comments.chunk_by(|a, b| a.level == b.level).collect();
        let mut roots = levels
            .first()
            .map(|level| vec![level.iter()])
            .unwrap_or_default();

        while let Some(root) = 'next: {
            while let Some(chunk) = roots.last_mut() {
                let root = chunk.next();

                if root.is_some() {
                    break 'next root;
                } else {
                    roots.pop();
                }
            }

            None
        } {
            let user =
                root.user_id.and_then(|id| users.get(&id).unwrap().clone());

            let comment = Comment::new(root.clone(), user);

            let path = if comment.level == 0 {
                result.push(comment);
                vec![result.len() - 1]
            } else {
                let mut path = vec![result.len() - 1];
                result.last_mut().unwrap().push(comment, &mut path);
                path
            };

            cache
                .comments
                .insert(root.id, this, path.into_boxed_slice());

            if let Some(level) = levels.get(roots.len()) {
                if let Some(children) = level
                    .chunk_by(|a, b| a.parent_id == b.parent_id)
                    .find(|chunk| {
                        chunk.first().and_then(|comment| comment.parent_id)
                            == Some(root.id)
                    })
                {
                    roots.push(children.iter());
                }
            }
        }

        Ok(result)
    }

    pub fn comment<F, R>(&self, path: &[usize], f: F) -> Option<R>
    where
        F: FnOnce(&mut Comment) -> R,
    {
        self.mutable.update(|post| post.comment(path, f)).flatten()
    }

    pub async fn comments<F, R>(
        &self,
        this: &Arc<Cached<Self>>,
        cache: &Cache,
        f: F,
    ) -> Result<R>
    where
        F: Fn(PostComments<'_>) -> R,
    {
        let result = self
            .mutable
            .map(|post| post.comments(self.id, &f))
            .found("post", self.id)?;

        if let Some(result) = result {
            return Ok(result);
        }

        let comments = self.build_comments(this, cache).await?;

        let result = f(PostComments {
            post: self.id,
            comments: &comments,
        });

        self.mutable.update(|post| {
            post.comments = Some(comments);
        });

        Ok(result)
    }

    pub fn delete(&self) {
        if let Some(post) = self.mutable.delete() {
            for object in &post.objects {
                object.delete_post(self.id);
            }

            if post.visibility != Visibility::Draft {
                if let Some(user) = &self.poster {
                    user.update(|user| user.post_count -= 1);
                }

                for tag in &post.tags {
                    tag.update(|tag| tag.post_count -= 1);
                }
            }
        }
    }

    pub fn delete_comment(
        &self,
        path: &[usize],
        recursive: bool,
    ) -> Vec<Comment> {
        self.mutable
            .update(|post| post.delete_comment(path, recursive))
            .unwrap_or_default()
    }

    pub fn delete_objects(&self, objects: &[Uuid], modified: DateTime) {
        self.mutable.update(|post| {
            post.objects.retain(|object| {
                let retain = !objects.contains(&object.id);

                if !retain {
                    object.delete_post(self.id);
                }

                retain
            });
            post.modified = modified;
        });
    }

    pub fn add_tag(&self, tag: Arc<Cached<Tag>>) {
        self.mutable.update(|post| post.add_tag(tag));
    }

    pub fn delete_tag(&self, id: Uuid) {
        self.mutable.update(|post| post.delete_tag(id));
    }

    pub fn publish(&self, timestamp: DateTime) {
        if let Some(user) = &self.poster {
            user.update(|user| user.post_count += 1);
        }

        self.mutable.update(|post| {
            for object in &post.objects {
                object.add_post(self.id);
            }

            for tag in &post.tags {
                tag.update(|tag| tag.post_count += 1);
            }

            post.visibility = Visibility::Public;
            post.created = timestamp;
            post.modified = timestamp;
        });
    }

    pub fn reply(&self, path: &[usize], reply: Comment) -> Option<usize> {
        self.mutable
            .update(|post| post.reply(path, reply))
            .flatten()
    }

    pub fn set_description(&self, description: String, modified: DateTime) {
        self.mutable.update(|post| {
            post.description = description;
            post.modified = modified
        });
    }

    pub fn set_related_posts(&self, posts: Vec<Uuid>) {
        self.mutable.update(|post| post.posts = posts);
    }

    pub fn set_title(&self, title: String, modified: DateTime) {
        self.mutable.update(|post| {
            post.title = title;
            post.modified = modified
        });
    }
}

impl Id for Post {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

pub struct PostComments<'a> {
    post: Uuid,
    comments: &'a Vec<Comment>,
}

impl<'a> PostComments<'a> {
    pub fn comment(&self, path: &[usize]) -> &Comment {
        let first = *path.first().expect("empty paths should not exist");
        let mut comment = self.comments.get(first).expect(EXISTING_COMMENT);

        for index in path[1..].iter().copied() {
            comment = comment.children.get(index).expect(EXISTING_COMMENT);
        }

        comment
    }

    pub fn get(&self, path: &[usize]) -> Option<minty::Comment> {
        let first = *path.first().expect("empty paths should not exist");
        let mut parent: Option<Uuid> = None;
        let mut comment = self.comments.get(first).expect(EXISTING_COMMENT);

        for index in path[1..].iter().copied() {
            parent = Some(comment.id);
            comment = comment.children.get(index).expect(EXISTING_COMMENT);
        }

        comment.model(self.post, parent)
    }

    pub fn get_all(&self) -> Vec<CommentData> {
        let mut result = Vec::new();

        self.comments
            .iter()
            .rev()
            .filter(|child| {
                !(child.content.is_empty() && child.children.is_empty())
            })
            .for_each(|comment| comment.thread(&mut result));

        result
    }

    pub fn user(&self, path: &[usize]) -> Option<Uuid> {
        self.comment(path).user.as_ref().map(|user| user.id)
    }
}

pub(super) struct CommentEntry(pub Arc<Cached<Post>>, pub Box<[usize]>);

pub(super) struct CommentPath {
    pub post: Weak<Cached<Post>>,
    pub path: Box<[usize]>,
}

pub(super) struct CommentMap {
    map: DashMap<Uuid, CommentPath>,
}

impl CommentMap {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    pub fn get(&self, id: Uuid) -> Option<CommentEntry> {
        let comment = self.map.get(&id)?;
        let post = comment.post.upgrade()?;
        let path = comment.path.clone();

        Some(CommentEntry(post, path))
    }

    fn delete_comment(&self, comment: &Comment) {
        self.map.remove(&comment.id);

        comment
            .children
            .iter()
            .for_each(|child| self.delete_comment(child));
    }

    pub fn delete(self: &Arc<Self>, comments: Vec<Comment>) {
        let this = self.clone();

        task::spawn_blocking(move || {
            comments
                .iter()
                .for_each(|comment| this.delete_comment(comment))
        });
    }

    pub fn insert(
        &self,
        id: Uuid,
        post: &Arc<Cached<Post>>,
        path: Box<[usize]>,
    ) {
        let post = Arc::downgrade(post);
        self.map.insert(id, CommentPath { post, path });
    }

    pub fn path(&self, id: Uuid) -> Box<[usize]> {
        self.map
            .get(&id)
            .expect("comment entry should exist in map")
            .path
            .clone()
    }
}

pub struct Posts<'a> {
    cache: &'a Cache,
}

impl<'a> Posts<'a> {
    pub(super) fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Arc<Cached<Post>>>> {
        self.cache
            .posts
            .get(id, || async {
                if let Some(post) = self.cache.database.read_post(id).await? {
                    Ok(Some(Post::new(post, self.cache, false).await?))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    async fn get_multiple(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Arc<Cached<Post>>>> {
        self.cache
            .posts
            .get_multiple(ids, |ids| async move {
                let posts = self.cache.database.read_posts(&ids).await?;
                let mut result = Vec::with_capacity(posts.len());

                for post in posts {
                    result.push(Post::new(post, self.cache, false).await?);
                }

                Ok(result)
            })
            .await
    }

    pub async fn previews(
        &self,
        ids: &[Uuid],
        user: Option<&Arc<Cached<User>>>,
    ) -> Result<Vec<PostPreview>> {
        Ok(self
            .cache
            .posts()
            .get_multiple(ids)
            .await?
            .into_iter()
            .filter_map(|post| post.preview(user))
            .collect())
    }

    pub async fn insert(&self, post: db::Post) -> Result<Arc<Cached<Post>>> {
        Ok(self
            .cache
            .posts
            .insert(Post::new(post, self.cache, true).await?))
    }

    pub fn remove(&self, post: &Arc<Cached<Post>>) {
        post.delete();
        self.cache.posts.remove(post.id);
    }
}
