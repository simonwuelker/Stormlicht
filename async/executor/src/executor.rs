use crate::{
    task::{Task, TaskID},
    waker::TaskWaker,
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

pub struct Executor {
    tasks: BTreeMap<TaskID, Task>,
    /// Tasks that can be progressed.
    /// A task has a waker, which can put the task into this
    /// queue.
    task_queue: Arc<Mutex<VecDeque<TaskID>>>,
    waker_cache: BTreeMap<TaskID, Waker>,
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            waker_cache: BTreeMap::new(),
        }
    }
}
impl Executor {
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        if self.tasks.insert(task.id, task).is_some() {
            panic!("Task has already been spawned");
        }

        self.task_queue.lock().unwrap().push_back(task_id);
    }

    fn next_task(&self) -> Option<TaskID> {
        self.task_queue.lock().unwrap().pop_front()
    }

    /// Poll each ready task once
    fn run_ready_tasks(&mut self) {
        while let Some(task_id) = self.next_task() {
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::create(task_id, Arc::clone(&self.task_queue)));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                },
                Poll::Pending => {},
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
        }
    }
}
