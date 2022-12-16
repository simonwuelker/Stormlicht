use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Poll, Context},
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskID(u64);

impl TaskID {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// An operation that can be completed asynchronously
pub struct Task {
    pub(crate) id: TaskID,
    task: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(task: Pin<Box<dyn Future<Output = ()> + 'static>>) -> Self {
        Self {
            id: TaskID::new(),
            task: task
        }
    }

    pub(crate) fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.task.as_mut().poll(context)

    }
}
