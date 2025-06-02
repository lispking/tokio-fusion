use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tracing::{debug, trace};

use crate::worker::{TaskEnvelope, WorkerState};

/// Worker-local task queue with work-stealing support
pub struct Worker {
    /// Worker ID for debugging
    id: usize,
    /// Local task queue (FIFO)
    queue: Mutex<std::collections::VecDeque<TaskEnvelope>>,
    /// Shared worker state
    state: Arc<WorkerState>,
}

impl Worker {
    /// Create a new worker
    pub(crate) fn new(id: usize, state: Arc<WorkerState>) -> Self {
        Self {
            queue: Mutex::new(std::collections::VecDeque::new()),
            id,
            state,
        }
    }

    /// Push a task to this worker's queue
    pub async fn push_task(&self, task: TaskEnvelope) {
        let mut queue = self.queue.lock().await;
        queue.push_back(task);
    }

    /// Try to steal a task from another worker
    pub async fn try_steal_task(&self) -> Option<TaskEnvelope> {
        let workers = self.state.workers.lock().await;
        let num_workers = workers.len();
        if num_workers <= 1 {
            return None; // No other workers to steal from
        }

        // Use thread-local RNG
        let start = rand::rng().random_range(0..num_workers);

        for i in 0..num_workers {
            let idx = (start + i) % num_workers;
            if idx == self.id {
                continue; // Don't steal from ourselves
            }

            if let Some(worker) = workers.get(idx) {
                if let Ok(mut queue) = worker.queue.try_lock() {
                    if let Some(task) = queue.pop_back() {
                        // Steal from the back (LIFO)
                        trace!("Worker {} stole a task from worker {}", self.id, idx);
                        return Some(task);
                    }
                }
            }
        }

        None
    }

    /// Try to get a task from the shared queue
    pub async fn try_get_shared_task(&self) -> Option<TaskEnvelope> {
        let mut shared_queue = self.state.shared_queue.lock().await;
        shared_queue.pop_front()
    }

    /// The main worker loop
    pub async fn run(&self) {
        tracing::info!("Worker {} started", self.id);

        loop {
            // 1. Try to get a task from local queue
            if let Some(task) = self.queue.lock().await.pop_front() {
                self.execute_task(task).await;
                continue;
            }

            // 2. Try to get a task from the shared queue
            if let Some(task) = self.try_get_shared_task().await {
                self.execute_task(task).await;
                continue;
            }

            // 3. Try to steal a task from another worker
            if let Some(task) = self.try_steal_task().await {
                self.execute_task(task).await;
                continue;
            }

            // 4. No tasks available, yield to avoid busy waiting
            tokio::task::yield_now().await;
        }
    }

    /// Execute a task
    async fn execute_task(&self, task: TaskEnvelope) {
        debug!(
            "Worker {} executing task with priority {}",
            self.id, task.priority
        );

        (task.task)();
        self.state.active_tasks.fetch_sub(1, Ordering::Relaxed);
    }
}

impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker").field("id", &self.id).finish()
    }
}
