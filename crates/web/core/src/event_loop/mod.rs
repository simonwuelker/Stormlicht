//! Handles concurrent events within the engine
//!
//! Refer to <https://html.spec.whatwg.org/multipage/webappapis.html#event-loops> for more information.

mod event_loop;
mod task;

pub use event_loop::{EventLoop, TaskSource};
pub use task::{MicroTask, Task};
