/// Returns the number of CPUs available to the current process
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}

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
            worker_threads: num_cpus(),
            queue_capacity: 1000,
        }
    }
}
