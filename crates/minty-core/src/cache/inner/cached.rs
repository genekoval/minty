use super::{Cache, Id};

use std::{
    ops::{Deref, DerefMut},
    sync::Weak,
};

#[derive(Debug)]
pub struct Cached<T: Id> {
    value: T,
    cache: Weak<Cache<T>>,
}

impl<T: Id> Cached<T> {
    pub fn new(value: T, cache: Weak<Cache<T>>) -> Self {
        Self { value, cache }
    }
}

impl<T: Id> Deref for Cached<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Id> DerefMut for Cached<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Id> Drop for Cached<T> {
    fn drop(&mut self) {
        if let Some(cache) = self.cache.upgrade() {
            cache.drop_value(self);
        }
    }
}

impl<T: Id> PartialEq for Cached<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: Id> Eq for Cached<T> {}
