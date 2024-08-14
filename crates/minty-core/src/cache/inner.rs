mod buffer;
mod cached;
mod policy;

pub use buffer::*;
pub use cached::Cached;

use policy::Policy;

use core::hash::Hash;
use dashmap::{mapref::entry::Entry, DashMap};
use log::trace;
use std::{
    fmt::{Debug, Display},
    future::Future,
    num::NonZeroUsize,
    sync::{Arc, Mutex, Weak},
};
use tokio::task;

pub trait Id: Debug + Send + Sync + 'static {
    type Id: Debug + Display + Copy + Hash + Eq + Send + Sync;

    fn id(&self) -> Self::Id;
}

#[derive(Debug)]
pub enum Event<T: Id> {
    Access(T::Id),
    None(T::Id),
    Insert(Arc<Cached<T>>),
}

enum CacheResult<T: Id> {
    Hit(Arc<Cached<T>>),
    Miss,
    None,
}

impl<T: Id> From<Option<Weak<Cached<T>>>> for CacheResult<T> {
    fn from(value: Option<Weak<Cached<T>>>) -> Self {
        match value.and_then(|weak| weak.upgrade()) {
            Some(value) => Self::Hit(value),
            None => Self::None,
        }
    }
}

impl<T: Id> From<CacheResult<T>> for Option<Arc<Cached<T>>> {
    fn from(value: CacheResult<T>) -> Self {
        match value {
            CacheResult::Hit(hit) => Some(hit),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Cache<T: Id> {
    token: BufferToken,
    name: String,
    map: DashMap<T::Id, Option<Weak<Cached<T>>>>,
    policy: Mutex<Policy<T>>,
    emit: EmitEvent<T>,
}

impl<T: Id> Cache<T> {
    pub fn new(name: &str, cap: NonZeroUsize, emit: EmitEvent<T>) -> Self {
        Self {
            token: BufferToken::new(),
            name: name.into(),
            map: DashMap::new(),
            policy: Mutex::new(Policy::new(cap)),
            emit,
        }
    }

    fn cached(self: &Arc<Self>, value: T) -> Arc<Cached<T>> {
        Arc::new(Cached::new(value, Arc::downgrade(self)))
    }

    fn commit(&self, events: Events<T>) {
        let mut policy = self.policy.lock().unwrap();

        events.for_each(|event| {
            if let Some(id) = policy.handle(event) {
                self.map.remove(&id);
            }
        });
    }

    fn emit(self: &Arc<Self>, event: Event<T>) {
        if let Some(events) = (self.emit)(&self.token, event) {
            let this = self.clone();

            task::spawn_blocking(move || {
                this.commit(events);
            });
        }
    }

    fn none(self: &Arc<Self>, id: T::Id) {
        self.map.insert(id, None);
        self.emit(Event::None(id));
    }

    fn _get(&self, id: T::Id) -> CacheResult<T> {
        self.map
            .get(&id)
            .map(|node| node.clone().into())
            .unwrap_or(CacheResult::Miss)
    }

    pub fn get_cached(&self, id: T::Id) -> Option<Arc<Cached<T>>> {
        self._get(id).into()
    }

    pub async fn get<F, Fut, E>(
        self: &Arc<Self>,
        id: T::Id,
        on_miss: F,
    ) -> Result<Option<Arc<Cached<T>>>, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Option<T>, E>>,
    {
        match self._get(id) {
            CacheResult::Hit(hit) => {
                self.emit(Event::Access(id));
                Ok(Some(hit))
            }
            CacheResult::None => {
                self.emit(Event::Access(id));
                Ok(None)
            }
            CacheResult::Miss => match on_miss().await? {
                Some(value) => Ok(Some(self.insert(value))),
                None => {
                    self.none(id);
                    Ok(None)
                }
            },
        }
    }

    pub async fn get_multiple<F, Fut, E>(
        self: &Arc<Self>,
        ids: &[T::Id],
        on_miss: F,
    ) -> Result<Vec<Arc<Cached<T>>>, E>
    where
        F: FnOnce(Vec<T::Id>) -> Fut,
        Fut: Future<Output = Result<Vec<T>, E>>,
    {
        let mut result = Vec::with_capacity(ids.len());
        let mut misses = Vec::new();

        for id in ids.iter().copied() {
            match self._get(id) {
                CacheResult::Hit(hit) => {
                    self.emit(Event::Access(id));
                    result.push(Ok(hit));
                }
                CacheResult::Miss => {
                    result.push(Err(id));
                    misses.push(id);
                }
                CacheResult::None => (),
            }
        }

        let mut misses = if misses.is_empty() {
            Default::default()
        } else {
            on_miss(misses).await?
        }
        .into_iter()
        .peekable();

        let result = result
            .into_iter()
            .filter_map(|item| match item {
                Ok(item) => Some(item),
                Err(id) => {
                    let next = misses.peek()?;

                    if next.id() == id {
                        Some(self.insert(misses.next().unwrap()))
                    } else {
                        self.none(id);
                        None
                    }
                }
            })
            .collect();

        Ok(result)
    }

    pub fn insert(self: &Arc<Self>, value: T) -> Arc<Cached<T>> {
        let entry = self.map.entry(value.id());

        if let Entry::Occupied(entry) = &entry {
            if let Some(value) = &entry.get() {
                if let Some(value) = value.upgrade() {
                    return value;
                }
            }
        }

        let cached = self.cached(value);
        entry.insert(Some(Arc::downgrade(&cached)));

        self.emit(Event::Insert(cached.clone()));

        cached
    }

    fn drop_value(&self, value: &Cached<T>) {
        self.map
            .remove_if(&value.id(), |_, v| {
                v.as_ref().is_some_and(|weak| weak.as_ptr() == value)
                    || v.is_none()
            })
            .inspect(|(k, _)| trace!("removed {} {}", self.name, k));
    }

    pub fn remove(&self, key: T::Id) {
        if let Entry::Occupied(mut entry) = self.map.entry(key) {
            *entry.get_mut() = None;
        }
    }
}
