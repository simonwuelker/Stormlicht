use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskID(u64);

impl TaskID {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// An operation that can be completed asynchronously
pub struct Task {
    id: TaskID,
    task: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    #[must_use]
    pub fn new(task: Pin<Box<dyn Future<Output = ()> + 'static>>) -> Self {
        Self {
            id: TaskID::new(),
            task: task,
        }
    }

    #[must_use]
    pub fn id(&self) -> TaskID {
        self.id
    }

    pub(crate) fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.task.as_mut().poll(context)
    }
}
