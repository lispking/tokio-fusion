use crate::config::ThreadPoolConfig;

/// A builder for creating a thread pool with custom configuration
pub struct ThreadPoolBuilder {
    config: ThreadPoolConfig,
}

impl ThreadPoolBuilder {
    /// Create a new thread pool builder
    pub fn new() -> Self {
        Self {
            config: ThreadPoolConfig::default(),
        }
    }

    /// Set the number of worker threads
    pub fn worker_threads(mut self, worker_threads: usize) -> Self {
        self.config.worker_threads = worker_threads;
        self
    }

    /// Set the capacity of the task queue
    pub fn queue_capacity(mut self, queue_capacity: usize) -> Self {
        self.config.queue_capacity = queue_capacity;
        self
    }

    /// Build the thread pool
    pub fn build(self) -> super::ThreadPool {
        super::ThreadPool::new(self.config)
    }
}

impl Default for ThreadPoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}
