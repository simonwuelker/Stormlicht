use crate::{
    dom::{dom_objects::Document, DomPtr},
    Window,
};

use super::{MicroTask, Task};

use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

fn global_event_loop() -> &'static Mutex<Vec<u8>> {
    static EVENT_LOOP: OnceLock<Mutex<EventLoop>> = OnceLock::new();
    EVENT_LOOP.get_or_init(|| Mutex::new(vec![]))
}

macro_rules! spin_event_loop_until {
    ($event_loop: ident, $condition: expr, $afterwards: block) => {
        // 1. Let task be the event loop's currently running task.
        let task = $event_loop.currently_running_task;

        // 2. Let task source be task's source.
        let task_source = task.source();

        // 3. FIXME: Let old stack be a copy of the JavaScript execution context stack.
        // 4. FIXME: Empty the JavaScript execution context stack.

        // 5. Perform a microtask checkpoint.
        $event_loop.perform_microtask_checkpoint();

        // 6. In parallel:
        let parallel_task = || {
            // 1. Wait until the condition goal is met.
            while !$condition {
                ::std::thread::sleep(::std::time::Duration::from_millis(50));
            }

            // 2. Queue a task on task source to:
            let queued_task = || {
                // 1. FIXME: Replace the JavaScript execution context stack with old stack.
                // 2. Perform any steps that appear after this spin the event loop instance in the original algorithm.
                $afterwards
            };
        };
        ::crate::infra::in_parallel(
            parallel_task,
            "\"Spin event loop until\" observer".to_string(),
        );

        // 7. Stop task, allowing whatever algorithm that invoked it to resume.
        return;
    };
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#generic-task-sources>
#[derive(Clone, Copy, Debug)]
pub enum TaskSource {
    /// <https://html.spec.whatwg.org/multipage/webappapis.html#dom-manipulation-task-source>
    DomManipulation,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#user-interaction-task-source>
    UserInteraction,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#networking-task-source>
    Networking,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#navigation-and-traversal-task-source>
    NavigationAndTraversal,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#rendering-task-source>
    Rendering,
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#event-loop>
#[derive(Clone)]
pub struct EventLoop {
    /// <https://html.spec.whatwg.org/multipage/webappapis.html#task-queue>
    task_queues: Vec<TaskQueue>,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#currently-running-task>
    currently_running_task: Option<Task>,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#microtask-queue>
    microtask_queue: VecDeque<MicroTask>,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#performing-a-microtask-checkpoint>
    is_performing_a_microtask_checkpoint: bool,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#last-render-opportunity-time>
    last_render_opportunity_time: Instant,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#last-idle-period-start-time>
    last_idle_period_start_time: Instant,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#same-loop-windows>
    same_loop_windows: Vec<Window>,

    is_window_event_loop: bool,
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#task-queue>
#[derive(Clone, Default)]
struct TaskQueue {
    tasks: Vec<Task>,
}

impl TaskQueue {
    pub fn append(&mut self, task: Task) {
        self.tasks.push(task);
    }
}

impl EventLoop {
    /// <https://html.spec.whatwg.org/multipage/webappapis.html#implied-event-loop>
    fn implied_event_loop() -> &'static mut Self {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#implied-document>
    fn implied_document(&self) -> Option<DomPtr<Document>> {
        // 1. If event loop is not a window event loop, then return null.
        if !self.is_window_event_loop() {
            return None;
        }

        todo!()
    }

    #[must_use]
    pub fn new_window_event_loop(window: Window) -> Self {
        let now = Instant::now();

        Self {
            task_queues: vec![TaskQueue::default()],
            currently_running_task: None,
            microtask_queue: VecDeque::default(),
            is_performing_a_microtask_checkpoint: false,
            last_render_opportunity_time: now,
            last_idle_period_start_time: now,
            is_window_event_loop: true,
            same_loop_windows: vec![window],
        }
    }

    #[must_use]
    pub fn task_queue_associated_with_source(&mut self, source: TaskSource) -> &mut TaskQueue {
        match source {
            TaskSource::DomManipulation => &mut self.task_queues[0],
            TaskSource::UserInteraction => &mut self.task_queues[1],
            TaskSource::Networking => &mut self.task_queues[2],
            TaskSource::NavigationAndTraversal => &mut self.task_queues[3],
            TaskSource::Rendering => &mut self.task_queues[4],
        }
    }

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-task>
    pub fn queue_task(
        source: TaskSource,
        steps: (),
        event_loop: Option<&mut Self>,
        document: Option<DomPtr<Document>>,
    ) {
        // 1. If event loop was not given, set event loop to the implied event loop.
        let event_loop = event_loop.unwrap_or_else(|| Self::implied_event_loop());

        // 2. If document was not given, set document to the implied document.
        let document = document.or_else(|| event_loop.implied_document());

        // 3. Let task be a new task.
        // 4. Set task's steps to steps.
        // 5. Set task's source to source.
        // 6. Set task's document to the document
        // 7. Set task's script evaluation environment settings object set to an empty set.
        let task = Task::new(steps, source, document);

        // 8. Let queue be the task queue to which source is associated on event loop.
        let queue = event_loop.task_queue_associated_with_source(source);

        // 9. Append task to queue.
        queue.append(task);
    }

    #[must_use]
    fn is_window_event_loop(&self) -> bool {
        self.is_window_event_loop
    }

    #[must_use]
    fn next_runnable_task(&mut self) -> Option<Task> {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model>
    pub fn run(&mut self) {
        // 1. Let oldestTask and taskStartTime be null.
        let mut oldest_task = None;
        let mut task_start_time = None;

        // 2. If the event loop has a task queue with at least one runnable task, then:
        //    1. Let taskQueue be one such task queue, chosen in an implementation-defined manner.
        if let Some(runnable_task) = self.next_runnable_task() {
            // 2. Set taskStartTime to the unsafe shared current time.
            task_start_time = Some(Instant::now());

            // 3. Set oldestTask to the first runnable task in taskQueue, and remove it from taskQueue.
            //    NOTE: We do this after we're done running the taks

            // 4. Set the event loop's currently running task to oldestTask.
            let runnable_task = self.currently_running_task.insert(runnable_task);

            // 5. Perform oldestTask's steps.
            runnable_task.run();

            // 6. Set the event loop's currently running task back to null.
            let runnable_task = self
                .currently_running_task
                .take()
                .expect("There must be a current task");

            // 7. Perform a microtask checkpoint.
            self.perform_microtask_checkpoint();

            oldest_task = Some(runnable_task);
        }

        // 3. Let now be the unsafe shared current time.
        let now = Instant::now();

        if let Some(oldest_task) = oldest_task.as_ref() {
            // FIXME: implement this part of the algorithm
            _ = now;
            _ = oldest_task;
            _ = task_start_time;
        }

        // 4. If this is a window event loop that has no runnable task in this event loop's task queues, then:
        if self.is_window_event_loop() && oldest_task.is_none() {
            // 1. Set this event loop's last idle period start time to the unsafe shared current time.
            self.last_idle_period_start_time = Instant::now();

            // 2. Let computeDeadline be the following steps:
            let compute_deadline = || {
                // 1. Let deadline be this event loop's last idle period start time plus 50.
                let deadline = self.last_idle_period_start_time + Duration::from_millis(50);

                // FIXME: Implement the rest of this algorithm (only relevant for animations)

                // 5. Return deadline.
                return deadline;
            };

            // 3. For each win of the same-loop windows for this event loop, perform the start an idle period algorithm
            //    for win with the following step: return the result of calling computeDeadline, coarsened given win's relevant settings
            //    object's cross-origin isolated capability.
            for window in &self.same_loop_windows {
                // FIXME: Coarsen the deadline here
                window.start_an_idle_period(compute_deadline);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint>
    fn perform_microtask_checkpoint(&mut self) {
        // 1. If the event loop's performing a microtask checkpoint is true, then return.
        if self.is_performing_a_microtask_checkpoint {
            return;
        }

        // 2. Set the event loop's performing a microtask checkpoint to true.
        self.is_performing_a_microtask_checkpoint = true;

        // 3. While the event loop's microtask queue is not empty:
        //    1. Let oldestMicrotask be the result of dequeuing from the event loop's microtask queue.
        while let Some(oldest_micro_task) = self.microtask_queue.pop_front() {
            // 2. Set the event loop's currently running task to oldestMicrotask.
            let oldest_micro_task = self.currently_running_task.insert(oldest_micro_task.into());

            // 3. Run oldestMicrotask.
            oldest_micro_task.run();

            // 4. Set the event loop's currently running task back to null.
            self.currently_running_task = None;
        }

        // FIXME: Implement steps 4-6

        // 7. Set the event loop's performing a microtask checkpoint to false.
        self.is_performing_a_microtask_checkpoint = false;
    }
}
