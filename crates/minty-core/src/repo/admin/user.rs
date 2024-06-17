use crate::{cache, error::Found, Cached, Repo, Result};

use std::sync::Arc;

pub struct User<'a> {
    repo: &'a Repo,
    user: Arc<Cached<cache::User>>,
}

impl<'a> User<'a> {
    pub(super) fn new(repo: &'a Repo, user: Arc<Cached<cache::User>>) -> Self {
        Self { repo, user }
    }

    pub async fn set_admin(&self, admin: bool) -> Result<()> {
        self.repo
            .database
            .update_admin(self.user.id, admin)
            .await?
            .found("user", self.user.id)?;

        self.user.update(|user| user.admin = admin);

        Ok(())
    }
}
