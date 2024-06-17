use crate::{cache, error::Found, Cached, Result};

use std::sync::Arc;

pub struct Tag {
    tag: Arc<Cached<cache::Tag>>,
}

impl Tag {
    pub(super) fn new(tag: Arc<Cached<cache::Tag>>) -> Self {
        Self { tag }
    }

    pub fn get(&self) -> Result<minty::Tag> {
        self.tag.model().found("tag", self.tag.id)
    }
}
