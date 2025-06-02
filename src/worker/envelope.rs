use std::fmt;

/// A task envelope that can be sent through channels
pub struct TaskEnvelope {
    /// The task to execute
    pub task: Box<dyn FnOnce() + Send + 'static>,
    /// The priority of the task
    pub priority: usize,
}

impl fmt::Debug for TaskEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskEnvelope")
            .field("priority", &self.priority)
            .finish()
    }
}
