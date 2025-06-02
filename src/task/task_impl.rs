use std::future::Future;

use futures::future::BoxFuture;

use crate::error::ThreadPoolResult;

/// A task that can be submitted to the thread pool
pub struct Task<T> {
    pub(crate) future: BoxFuture<'static, ThreadPoolResult<T>>,
    pub(crate) priority: usize,
}

impl<T: Send + 'static> Task<T> {
    /// Create a new task with the given future and priority
    pub fn new<F>(future: F, priority: usize) -> Self
    where
        F: Future<Output = ThreadPoolResult<T>> + Send + 'static,
    {
        Self {
            future: Box::pin(future),
            priority,
        }
    }

    /// Create a new task with the given closure and priority
    pub fn from_fn<F>(f: F, priority: usize) -> Self
    where
        F: FnOnce() -> ThreadPoolResult<T> + Send + 'static,
    {
        Self::new(async move { f() }, priority)
    }
}
