mod comment;
mod object;
mod post;
mod posts;
mod tag;
mod tags;
mod user;
mod users;

pub use comment::*;
pub use object::*;
pub use post::*;
pub use posts::*;
pub use tag::*;
pub use tags::*;
pub use user::*;
pub use users::*;

use crate::{cache, error::Found, About, Cached, Repo, Result};

use minty::Uuid;
use std::sync::Arc;

pub struct OptionalUser<'a> {
    repo: &'a Repo,
    user: Option<Arc<Cached<cache::User>>>,
}

impl<'a> OptionalUser<'a> {
    pub(super) fn new(
        repo: &'a Repo,
        user: Option<Arc<Cached<cache::User>>>,
    ) -> Self {
        Self { repo, user }
    }

    pub fn about(self) -> About {
        self.repo.about()
    }

    pub fn comment(self, id: Uuid) -> Comment<'a> {
        Comment::new(self.repo, id)
    }

    pub async fn post(self, id: Uuid) -> Result<Post<'a>> {
        let post = self.repo.cache.posts().get(id).await?.found("post", id)?;
        Post::new(self.repo, self.user, post)
    }

    pub fn posts(self) -> Posts<'a> {
        Posts::new(self.repo, self.user)
    }

    pub fn object(self, id: Uuid) -> Object<'a> {
        Object::new(self.repo, id)
    }

    pub async fn other(self, user: Uuid) -> Result<User> {
        let user = self
            .repo
            .cache
            .users()
            .get(user)
            .await?
            .found("user", user)?;

        Ok(User::new(user))
    }

    pub async fn tag(self, id: Uuid) -> Result<Tag> {
        let tag = self.repo.cache.tags().get(id).await?.found("tag", id)?;
        Ok(Tag::new(tag))
    }

    pub fn tags(self) -> Tags<'a> {
        Tags::new(self.repo)
    }

    pub fn users(self) -> Users<'a> {
        Users::new(self.repo)
    }
}
