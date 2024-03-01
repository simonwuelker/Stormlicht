use std::time::Instant;

/// <https://html.spec.whatwg.org/multipage/nav-history-apis.html#window>
#[derive(Clone)]
pub struct Window {}

impl Window {
    /// <https://w3c.github.io/requestidlecallback/#start-an-idle-period-algorithm>
    pub fn start_an_idle_period<F>(&self, get_deadline: F)
    where
        F: Fn() -> Instant,
    {
        // 1. Optionally, if the user agent determines the idle period should be delayed, return from this algorithm.

        // 2. Let pending_list be window's list of idle request callbacks.
        let pending_lest = self.list_of_idle_request_callbacks();

        // 3. Let run_list be window's list of runnable idle callbacks.
        let run_list = self.list_of_runnable_idle_callbacks();

        // 4. Append all entries from pending_list into run_list preserving order.
        run_list.extend(pending_list);

        // 5. Clear pending_list.
        pending_list.clear();

        // 6. Queue a task on the queue associated with the idle-task task source,
        //    which performs the steps defined in the invoke idle callbacks algorithm
        //    with window and getDeadline as parameters.
    }
}
