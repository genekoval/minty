use super::{Cached, Event, Id};

use lru::LruCache;
use std::{num::NonZeroUsize, sync::Arc};

#[derive(Debug)]
pub struct Policy<T: Id>(LruCache<T::Id, Option<Arc<Cached<T>>>>);

impl<T: Id> Policy<T> {
    pub fn new(cap: NonZeroUsize) -> Self {
        Self(LruCache::new(cap))
    }

    pub fn handle(&mut self, event: Event<T>) -> Option<T::Id> {
        match event {
            Event::Access(id) => {
                self.0.promote(&id);
                None
            }
            Event::None(id) => self.0.push(id, None),
            Event::Insert(value) => self.0.push(value.id(), Some(value)),
        }
        .and_then(
            |(id, value)| {
                if value.is_none() {
                    Some(id)
                } else {
                    None
                }
            },
        )
    }
}
