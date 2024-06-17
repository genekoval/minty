mod objects;
mod user;

pub use objects::Objects;
pub use user::User;

use crate::{cache, error::Found, Cached, Repo, Result};

use minty::Uuid;
use std::sync::Arc;

pub struct Admin<'a> {
    repo: &'a Repo,
}

impl<'a> Admin<'a> {
    pub(super) fn new(
        repo: &'a Repo,
        admin: Arc<Cached<cache::User>>,
    ) -> Result<Self> {
        admin.deny_permission()?;
        Ok(Self { repo })
    }

    pub fn objects(self) -> Objects<'a> {
        Objects::new(self.repo)
    }

    pub async fn user(self, id: Uuid) -> Result<User<'a>> {
        let user = self.repo.cache.users().get(id).await?.found("user", id)?;
        Ok(User::new(self.repo, user))
    }
}
