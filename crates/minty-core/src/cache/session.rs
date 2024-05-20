use crate::SessionId;

use lru::LruCache;
use minty::Uuid;
use std::{num::NonZeroUsize, sync::Mutex};

pub struct SessionStore {
    cache: Mutex<LruCache<SessionId, Uuid>>,
}

impl SessionStore {
    pub fn new(cap: NonZeroUsize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(cap)),
        }
    }

    pub fn get(&self, session: SessionId) -> Option<Uuid> {
        self.cache.lock().unwrap().get(&session).copied()
    }

    pub fn insert(&self, session: SessionId, user: Uuid) {
        self.cache.lock().unwrap().push(session, user);
    }

    pub fn remove(&self, session: SessionId) {
        self.cache.lock().unwrap().pop(&session);
    }
}
