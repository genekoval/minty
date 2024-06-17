use crate::{cache, error::Found, Cached, Result};

use std::sync::Arc;

pub struct User {
    user: Arc<Cached<cache::User>>,
}

impl User {
    pub(super) fn new(user: Arc<Cached<cache::User>>) -> Self {
        Self { user }
    }

    pub fn get(&self) -> Result<minty::User> {
        self.user.model().found("user", self.user.id)
    }
}
