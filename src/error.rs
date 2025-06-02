use serde_json;
use thiserror::Error;

/// Thread pool error type
#[derive(Error, Debug)]
pub enum ThreadPoolError {
    /// Failed to submit a task to the thread pool
    #[error("Task submission failed: {0}")]
    TaskSubmissionFailed(String),

    /// Task execution failed with the given error message
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    /// The thread pool is shutting down
    #[error("Thread pool is shutting down")]
    ShuttingDown,

    /// The task was cancelled before completion
    #[error("Task was cancelled")]
    TaskCancelled,

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for ThreadPoolError {
    fn from(err: serde_json::Error) -> Self {
        ThreadPoolError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for ThreadPoolError {
    fn from(err: std::io::Error) -> Self {
        ThreadPoolError::TaskExecutionFailed(err.to_string())
    }
}

/// Result type for the thread pool service
pub type ThreadPoolResult<T> = Result<T, ThreadPoolError>;
