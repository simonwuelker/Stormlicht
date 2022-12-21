use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    task::{Wake, Waker},
};

use crate::task::TaskID;

pub(crate) struct TaskWaker {
    task_id: TaskID,
    task_queue: Arc<Mutex<VecDeque<TaskID>>>,
}

impl TaskWaker {
    pub(crate) fn new(task_id: TaskID, task_queue: Arc<Mutex<VecDeque<TaskID>>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
    fn wake_task(&self) {
        self.task_queue.lock().unwrap().push_back(self.task_id);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
