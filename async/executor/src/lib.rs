mod task;
mod executor;
mod waker;

pub use executor::Executor;
pub use task::Task;




#[cfg(test)]
mod tests {
    use super::task::*;
    use super::executor::*;

    #[test]
    fn test_basic_async() {
        async fn number() -> i32 {
            42
        }

        async fn async_fn() {
            assert_eq!(number().await, 42);
        }

        let mut executor = Executor::new();
        executor.spawn(Task::new(Box::pin(async_fn())));
        executor.run();
    }
}
