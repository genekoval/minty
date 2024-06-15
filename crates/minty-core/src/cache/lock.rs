use std::sync::RwLock;

#[derive(Debug)]
pub struct CacheLock<T>(RwLock<Option<T>>);

impl<T> CacheLock<T> {
    pub fn new(data: T) -> Self {
        Self(RwLock::new(Some(data)))
    }

    pub fn delete(&self) {
        self.0.write().unwrap().take();
    }

    pub fn is_deleted(&self) -> bool {
        self.0.read().unwrap().is_none()
    }

    pub fn map<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.0.read().unwrap().as_ref().map(f)
    }

    pub fn update<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.0.write().unwrap().as_mut().map(f)
    }
}
