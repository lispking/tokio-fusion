use rand::Rng;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::{Mutex as TokioMutex, mpsc, oneshot};

use crate::{
    ThreadPoolError,
    config::ThreadPoolConfig,
    error::ThreadPoolResult,
    task::{Task, TaskHandle},
    worker::{TaskEnvelope, Worker, WorkerState},
};

/// A high-performance thread pool service based on Tokio
pub struct ThreadPool {
    sender: mpsc::Sender<TaskEnvelope>,
    active_tasks: Arc<AtomicUsize>,
    config: ThreadPoolConfig,
}

impl ThreadPool {
    /// Create a new thread pool with the given configuration
    pub fn new(config: ThreadPoolConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.queue_capacity);
        let active_tasks = Arc::new(AtomicUsize::new(0));

        Self::spawn_workers(receiver, config.worker_threads, Arc::clone(&active_tasks));

        Self {
            sender,
            active_tasks,
            config,
        }
    }

    /// Submit a task to the thread pool
    pub async fn submit<T>(&self, task: Task<T>) -> ThreadPoolResult<TaskHandle<T>>
    where
        T: Send + 'static,
    {
        let (sender, receiver) = oneshot::channel();
        let priority = task.priority;
        let future = task.future;

        // Create a task that will execute the future and send the result
        let task_fn = Box::new(move || {
            tokio::spawn(async move {
                let result = future.await;
                let _ = sender.send(result);
            });
        });

        let envelope = TaskEnvelope {
            task: task_fn,
            priority,
        };

        self.sender
            .send(envelope)
            .await
            .map_err(|_| ThreadPoolError::ShuttingDown)?;

        self.active_tasks.fetch_add(1, Ordering::SeqCst);

        Ok(TaskHandle { receiver })
    }

    /// Get the number of active tasks
    pub fn active_tasks(&self) -> usize {
        self.active_tasks.load(Ordering::SeqCst)
    }

    /// Get the configuration
    pub fn config(&self) -> &ThreadPoolConfig {
        &self.config
    }

    /// Spawn worker threads to process tasks
    fn spawn_workers(
        receiver: mpsc::Receiver<TaskEnvelope>,
        num_workers: usize,
        active_tasks: Arc<AtomicUsize>,
    ) {
        let shared_queue = Arc::new(TokioMutex::new(std::collections::VecDeque::new()));
        let workers = Arc::new(TokioMutex::new(Vec::new()));

        // Create worker state
        let worker_state = Arc::new(WorkerState {
            workers: Arc::clone(&workers),
            shared_queue: Arc::clone(&shared_queue),
            active_tasks: Arc::clone(&active_tasks),
        });

        // Create workers
        for worker_id in 0..num_workers {
            let worker = Worker::new(worker_id, Arc::clone(&worker_state));
            let worker_arc = Arc::new(worker);

            // Clone the workers Arc for this iteration
            let workers_clone = Arc::clone(&workers);
            let worker_clone = Arc::clone(&worker_arc);
            tokio::spawn(async move {
                let mut workers = workers_clone.lock().await;
                workers.push(worker_clone);
            });
            // Start worker task
            let worker_runner = Arc::clone(&worker_arc);
            tokio::spawn(async move {
                worker_runner.run().await;
            });
        }

        // Spawn a task to distribute tasks from the receiver to workers
        let workers_for_dist = Arc::clone(&workers);
        tokio::spawn(async move {
            let mut receiver = receiver;
            while let Some(task) = receiver.recv().await {
                // Get a random worker
                let workers = workers_for_dist.lock().await;
                let worker_count = workers.len();
                if worker_count == 0 {
                    continue;
                }
                let worker_id = rand::rng().random_range(0..worker_count);
                if let Some(worker) = workers.get(worker_id) {
                    worker.push_task(task).await;
                }
            }
        });
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new(ThreadPoolConfig::default())
    }
}
