mod comment;
mod comments;
mod conf;
mod inner;
mod lock;
mod objects;
mod posts;
mod sessions;
mod tags;
mod users;

pub use comment::*;
pub use comments::*;
pub use conf::Config;
pub use inner::{Cached, Id};
pub use objects::*;
pub use posts::*;
pub use sessions::*;
pub use tags::*;
pub use users::*;

use inner::{BufferStorage, BufferToken, Event, Events};
use lock::CacheLock;

use crate::{db::Database, obj::Bucket, Result};

use std::sync::Arc;

mod buffer {
    use super::*;

    use std::cell::RefCell;

    thread_local! {
        static OBJECTS: RefCell<BufferStorage<Object>> =
            RefCell::new(BufferStorage::new());

        static POSTS: RefCell<BufferStorage<Post>> =
            RefCell::new(BufferStorage::new());

        static SESSIONS: RefCell<BufferStorage<Session>> =
            RefCell::new(BufferStorage::new());

        static TAGS: RefCell<BufferStorage<Tag>> =
            RefCell::new(BufferStorage::new());

        static USERS: RefCell<BufferStorage<User>> =
            RefCell::new(BufferStorage::new());

    }

    macro_rules! buffer_fn {
        ($f:ident, $t:ty, $var:ident) => {
            pub fn $f(
                token: &BufferToken,
                event: Event<$t>,
            ) -> Option<Events<$t>> {
                $var.with_borrow_mut(|storage| storage.emit(token, event))
            }
        };
    }

    buffer_fn!(objects, Object, OBJECTS);
    buffer_fn!(posts, Post, POSTS);
    buffer_fn!(sessions, Session, SESSIONS);
    buffer_fn!(tags, Tag, TAGS);
    buffer_fn!(users, User, USERS);
}

pub struct Cache {
    database: Database,
    bucket: Bucket,
    comments: Arc<CommentMap>,
    objects: Arc<inner::Cache<Object>>,
    posts: Arc<inner::Cache<Post>>,
    sessions: Arc<inner::Cache<Session>>,
    tags: Arc<inner::Cache<Tag>>,
    users: Arc<inner::Cache<User>>,
}

impl Cache {
    pub fn new(database: Database, bucket: Bucket, config: &Config) -> Self {
        Self {
            database,
            bucket,
            comments: Arc::new(CommentMap::new()),
            objects: Arc::new(inner::Cache::new(
                "object",
                config.objects(),
                buffer::objects,
            )),
            posts: Arc::new(inner::Cache::new(
                "post",
                config.posts(),
                buffer::posts,
            )),
            sessions: Arc::new(inner::Cache::new(
                "session",
                config.sessions(),
                buffer::sessions,
            )),
            tags: Arc::new(inner::Cache::new(
                "tag",
                config.tags(),
                buffer::tags,
            )),
            users: Arc::new(inner::Cache::new(
                "user",
                config.users(),
                buffer::users,
            )),
        }
    }

    pub fn comments(&self) -> Comments {
        Comments::new(self)
    }

    pub fn objects(&self) -> Objects {
        Objects::new(self)
    }

    pub fn posts(&self) -> Posts {
        Posts::new(self)
    }

    pub fn sessions(&self) -> Sessions {
        Sessions::new(self)
    }

    pub fn tags(&self) -> Tags {
        Tags::new(self)
    }

    pub fn users(&self) -> Users {
        Users::new(self)
    }
}
