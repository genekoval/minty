mod comment;
mod edit;
mod objects;
mod post;
mod posts;
mod tag;
mod tags;

pub use comment::*;
pub use edit::Edit;
pub use objects::*;
pub use post::*;
pub use posts::*;
pub use tag::*;
pub use tags::*;

use crate::{
    cache, error::Found, model::Invitation, Cached, Repo, Result, SessionId,
    SessionInfo,
};

use chrono::{Duration, Local};
use minty::Uuid;
use std::sync::Arc;

pub struct WithUser<'a> {
    repo: &'a Repo,
    user: Arc<Cached<cache::User>>,
}

impl<'a> WithUser<'a> {
    pub(super) fn new(repo: &'a Repo, user: Arc<Cached<cache::User>>) -> Self {
        Self { repo, user }
    }

    pub fn comment(self, id: Uuid) -> Comment<'a> {
        Comment::new(self.repo, self.user, id)
    }

    pub async fn create_session(self) -> Result<SessionInfo> {
        let user_id = self.user.id;
        let session = SessionId::generate();
        let digest = session.digest();
        let max_age = Duration::days(30);
        let expiration = Local::now() + max_age;

        self.repo
            .database
            .create_user_session(self.user.id, &digest, expiration)
            .await?;

        self.repo
            .cache
            .sessions()
            .insert(digest, self.user, expiration);

        Ok(SessionInfo {
            id: session,
            user_id,
            max_age,
        })
    }

    pub fn edit_self(self) -> Edit<'a> {
        Edit::new(self.repo, self.user)
    }

    pub fn get_self(&self) -> Result<minty::User> {
        self.user.model().found("user", self.user.id)
    }

    pub fn invite(&self) -> Result<String> {
        let exp = Duration::days(1);
        let invitation = Invitation::new(self.user.id);

        self.repo.auth.encode_jwt(exp, invitation)
    }

    pub fn objects(self) -> Objects<'a> {
        Objects::new(self.repo)
    }

    pub async fn post(self, id: Uuid) -> Result<Post<'a>> {
        let post = self.repo.cache.posts().get(id).await?.found("post", id)?;
        Ok(Post::new(self.repo, self.user, post))
    }

    pub fn posts(self) -> Posts<'a> {
        Posts::new(self.repo, self.user)
    }

    pub async fn tag(self, id: Uuid) -> Result<Tag<'a>> {
        let tag = self.repo.cache.tags().get(id).await?.found("tag", id)?;
        tag.can_edit(&self.user)?;
        Ok(Tag::new(self.repo, tag))
    }

    pub fn tags(self) -> Tags<'a> {
        Tags::new(self.repo, self.user)
    }
}
