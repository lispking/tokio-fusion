use std::sync::Arc;

use tokio::sync::Mutex;

mod worker_impl;
pub use worker_impl::Worker;

mod envelope;
pub use envelope::TaskEnvelope;

/// Worker-local task queue with work-stealing support
pub(crate) struct WorkerState {
    /// Reference to other workers for work stealing
    pub workers: Arc<Mutex<Vec<Arc<Worker>>>>,
    /// Reference to shared task queue for stealing
    pub shared_queue: Arc<Mutex<std::collections::VecDeque<TaskEnvelope>>>,
    /// Number of active tasks
    pub active_tasks: Arc<std::sync::atomic::AtomicUsize>,
}
