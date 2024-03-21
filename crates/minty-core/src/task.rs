use chrono::{Duration, Local};
use minty::DateTime;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Default)]
struct Inner {
    token: CancellationToken,
    start: DateTime,
    end: RwLock<Option<DateTime>>,
    total: u64,
    completed: AtomicU64,
    errors: AtomicU64,
}

#[derive(Clone, Debug)]
pub struct Task {
    inner: Arc<Inner>,
}

impl Task {
    pub(crate) fn new(total: u64) -> Self {
        Self {
            inner: Arc::new(Inner {
                start: Local::now(),
                total,
                ..Default::default()
            }),
        }
    }

    pub fn guard(&self) -> TaskGuard {
        TaskGuard::new(self.clone())
    }

    pub fn cancel(&self) {
        self.inner.token.cancel()
    }

    pub async fn cancelled(&self) {
        self.inner.token.cancelled().await
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.inner.token.clone()
    }

    pub fn started(&self) -> DateTime {
        self.inner.start
    }

    pub fn ended(&self) -> Option<DateTime> {
        *self.inner.end.read().unwrap()
    }

    pub fn elapsed(&self) -> Duration {
        self.ended().unwrap_or_else(Local::now) - self.started()
    }

    pub fn is_finished(&self) -> bool {
        self.ended().is_some()
    }

    pub fn total(&self) -> u64 {
        self.inner.total
    }

    pub fn completed(&self) -> u64 {
        self.inner.completed.load(Ordering::Relaxed)
    }

    pub fn errors(&self) -> u64 {
        self.inner.errors.load(Ordering::Relaxed)
    }

    pub(crate) fn increment(&self) {
        self.inner.completed.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn error(&self) {
        self.inner.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn finish(&self) {
        *self.inner.end.write().unwrap() = Some(Local::now());
        self.inner.token.cancel();
    }
}

pub struct TaskGuard {
    task: Task,
}

impl TaskGuard {
    fn new(task: Task) -> Self {
        Self { task }
    }

    pub fn task(&self) -> Task {
        self.task.clone()
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.task.finish();
    }
}
