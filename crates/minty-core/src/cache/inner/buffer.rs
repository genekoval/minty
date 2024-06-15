use super::{Event, Id};

use std::{
    mem::{self, MaybeUninit},
    sync::Mutex,
};

const BUFFER_LEN: usize = 64;

#[derive(Debug)]
pub struct Events<T: Id>([MaybeUninit<Event<T>>; BUFFER_LEN]);

impl<T: Id> Events<T> {
    pub fn for_each<F>(self, f: F)
    where
        F: FnMut(Event<T>),
    {
        self.0
            .into_iter()
            .map(|event| unsafe { event.assume_init() })
            .for_each(f);
    }
}

#[derive(Debug)]
pub struct Buffer<T: Id> {
    data: [MaybeUninit<Event<T>>; BUFFER_LEN],
    len: usize,
}

impl<T: Id> Buffer<T> {
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.data[0..self.len].iter_mut().for_each(|item| unsafe {
            mem::replace(item, MaybeUninit::uninit()).assume_init();
        });

        self.len = 0;
    }

    fn push(&mut self, event: Event<T>) -> Option<Events<T>> {
        let result = if self.len == self.data.len() {
            self.len = 0;
            Some(self.take())
        } else {
            None
        };

        self.data[self.len].write(event);
        self.len += 1;

        result
    }

    fn take(&mut self) -> Events<T> {
        let new = unsafe { MaybeUninit::uninit().assume_init() };
        Events(mem::replace(&mut self.data, new))
    }
}

pub struct VersionedBuffer<T: Id> {
    buffer: Buffer<T>,
    version: usize,
}

impl<T: Id> VersionedBuffer<T> {
    fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            version: 0,
        }
    }

    fn get(&mut self, version: usize) -> &mut Buffer<T> {
        if version != self.version {
            self.buffer.clear();
            self.version = version;
        }

        &mut self.buffer
    }
}

pub struct BufferStorage<T: Id>(Vec<VersionedBuffer<T>>);

impl<T: Id> BufferStorage<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn emit(
        &mut self,
        token: &BufferToken,
        event: Event<T>,
    ) -> Option<Events<T>> {
        if let Some(buffer) = self.0.get_mut(token.index) {
            return buffer.get(token.version).push(event);
        }

        for _ in self.0.len()..=token.index {
            self.0.push(VersionedBuffer::new());
        }

        self.0.last_mut().unwrap().get(token.version).push(event)
    }
}

struct BufferInfo {
    version: usize,
    claimed: bool,
}

impl BufferInfo {
    fn new() -> Self {
        Self {
            version: 0,
            claimed: true,
        }
    }

    fn claim(&mut self) -> usize {
        assert!(!self.claimed);

        self.claimed = true;
        self.version += 1;

        self.version
    }

    fn release(&mut self) {
        self.claimed = false;
    }
}

static INFO: Mutex<Vec<BufferInfo>> = Mutex::new(Vec::new());

#[derive(Debug)]
pub struct BufferToken {
    index: usize,
    version: usize,
}

impl BufferToken {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for BufferToken {
    fn default() -> Self {
        let mut info = INFO.lock().unwrap();

        info.iter_mut()
            .enumerate()
            .find_map(|(index, info)| {
                if info.claimed {
                    None
                } else {
                    Some(Self {
                        index,
                        version: info.claim(),
                    })
                }
            })
            .unwrap_or_else(|| {
                let index = info.len();
                info.push(BufferInfo::new());

                Self { index, version: 0 }
            })
    }
}

impl Drop for BufferToken {
    fn drop(&mut self) {
        if let Some(info) = INFO.lock().unwrap().get_mut(self.index) {
            info.release();
        }
    }
}

pub type EmitEvent<T> = fn(&BufferToken, Event<T>) -> Option<Events<T>>;
