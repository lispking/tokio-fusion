use std::sync::Arc;

use futures::stream::{FuturesUnordered, Stream, StreamExt};

use crate::{ThreadPoolError, error::ThreadPoolResult, pool::ThreadPool, task::Task};

/// A batch executor for running multiple tasks and collecting their results
pub struct BatchExecutor<T> {
    thread_pool: Arc<ThreadPool>,
    tasks: Vec<Task<T>>,
}

impl<T: Send + 'static> BatchExecutor<T> {
    /// Create a new batch executor with the given thread pool
    pub fn new(thread_pool: Arc<ThreadPool>) -> Self {
        Self {
            thread_pool,
            tasks: Vec::new(),
        }
    }

    /// Add a task to the batch
    pub fn add_task<F>(&mut self, future: F, priority: usize)
    where
        F: std::future::Future<Output = ThreadPoolResult<T>> + Send + 'static,
    {
        self.tasks.push(Task::new(future, priority));
    }

    /// Add a task from a closure to the batch
    pub fn add_task_fn<F>(&mut self, f: F, priority: usize)
    where
        F: FnOnce() -> ThreadPoolResult<T> + Send + 'static,
    {
        self.tasks.push(Task::from_fn(f, priority));
    }

    /// Execute all tasks and collect their results
    pub async fn execute(self) -> Vec<ThreadPoolResult<T>> {
        let mut handles = Vec::with_capacity(self.tasks.len());

        for task in self.tasks {
            match self.thread_pool.submit(task).await {
                Ok(handle) => handles.push(handle),
                Err(e) => {
                    tracing::error!("Failed to submit task: {}", e);
                }
            }
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(handle.await_result().await);
        }

        results
    }

    /// Execute all tasks and return a stream of results as they complete
    pub fn execute_stream(self) -> impl Stream<Item = ThreadPoolResult<T>> {
        let handles = FuturesUnordered::new();

        for task in self.tasks {
            let thread_pool = Arc::clone(&self.thread_pool);
            handles.push(tokio::spawn(async move {
                match thread_pool.submit(task).await {
                    Ok(handle) => handle.await_result().await,
                    Err(e) => Err(e.into()),
                }
            }));
        }

        handles.map(|result| match result {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(ThreadPoolError::TaskExecutionFailed(e.to_string())),
        })
    }
}
