//! <https://html.spec.whatwg.org/multipage/webappapis.html#concept-task>

use std::ops::{Deref, DerefMut};

use crate::dom::{dom_objects::Document, DomPtr};

use super::TaskSource;

/// <https://html.spec.whatwg.org/multipage/webappapis.html#concept-task>
#[derive(Clone)]
pub struct Task {
    /// <https://html.spec.whatwg.org/multipage/webappapis.html#concept-task-steps>
    steps: (),

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#concept-task-source>
    source: TaskSource,

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#concept-task-document>
    document: Option<DomPtr<Document>>,
}

impl Task {
    #[must_use]
    pub fn new(steps: (), source: TaskSource, document: Option<DomPtr<Document>>) -> Self {
        Self {
            steps,
            source,
            document,
        }
    }

    #[must_use]
    pub fn source(&self) -> TaskSource {
        self.source
    }

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#concept-task-runnable>
    #[must_use]
    pub fn is_runnable(&self) -> bool {
        // A task is runnable if its document is either null or fully active.
        !self
            .document
            .as_ref()
            .is_some_and(|d| !d.borrow().is_fully_active())
    }

    /// Perform the tasks steps
    pub fn run(&self) {}
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#microtask>
#[derive(Clone)]
pub struct MicroTask(Task);

impl Deref for MicroTask {
    type Target = Task;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MicroTask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<MicroTask> for Task {
    fn from(value: MicroTask) -> Self {
        value.0
    }
}
