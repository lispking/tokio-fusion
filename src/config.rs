/// Configuration for the thread pool
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// The number of worker threads to spawn
    pub worker_threads: usize,
    /// The capacity of the task queue
    pub queue_capacity: usize,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            queue_capacity: 1000,
        }
    }
}
