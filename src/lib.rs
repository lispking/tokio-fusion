//! # Tokio Fusion
//!
//! A high-performance thread pool service built on top of Tokio, providing an easy-to-use API
//! for asynchronous task execution with work-stealing capabilities.
//!
//! ## Features
//!
//! - **High Performance**: Built on top of Tokio's efficient runtime
//! - **Flexible Task Submission**: Submit individual tasks or batches
//! - **Priority Support**: Assign priorities to tasks
//! - **Streaming Results**: Stream results as they complete
//! - **Configurable**: Customize worker threads and queue capacity
//! - **Error Handling**: Comprehensive error types and handling
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio-fusion = "0.1"
//! ```
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```no_run
//! use std::sync::Arc;
//! use tokio_fusion::{ThreadPool, Task, ThreadPoolResult};
//!
//! async fn my_task(id: usize) -> ThreadPoolResult<String> {
//!     // Your async task logic here
//!     Ok(format!("Result from task {}", id))
//! }
//!
//! #[tokio::main]
//! async fn main() -> ThreadPoolResult<()> {
//!     // Create a thread pool with default configuration
//!     let thread_pool = Arc::new(ThreadPool::default());
//!     
//!     // Create and submit a task
//!     let task = Task::new(my_task(1), 1);
//!     let handle = thread_pool.submit(task).await?;
//!     
//!     // Wait for the result
//!     let result = handle.await_result().await?;
//!     println!("Task result: {}", result);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Batch Execution
//!
//! ```no_run
//! use std::sync::Arc;
//! use tokio_fusion::{ThreadPool, BatchExecutor, ThreadPoolResult};
//! use futures::stream::StreamExt;
//!
//! async fn my_task(id: usize) -> ThreadPoolResult<String> {
//!     // Your async task logic here
//!     Ok(format!("Result from task {}", id))
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let thread_pool = Arc::new(ThreadPool::default());
//!     let mut batch = BatchExecutor::new(Arc::clone(&thread_pool));
//!     
//!     // Add tasks to the batch
//!     for i in 0..5 {
//!         batch.add_task(my_task(i), i);
//!     }
//!     
//!     // Stream results as they complete
//!     let mut stream = batch.execute_stream();
//!     while let Some(result) = stream.next().await {
//!         println!("Got result: {:?}", result);
//!     }
//! }
//! ```

/// Batch task execution functionality.
///
/// This module provides the `BatchExecutor` type for executing multiple tasks
/// as a batch and collecting their results either all at once or as a stream.
pub mod batch;

/// Thread pool configuration types and utilities.
///
/// This module contains the `ThreadPoolConfig` struct for configuring
/// the behavior of the thread pool.
pub mod config;

/// Error types for the thread pool.
///
/// This module defines the `ThreadPoolError` enum which represents
/// the various error conditions that can occur when using the thread pool.
pub mod error;

/// Thread pool implementation.
///
/// This module contains the main `ThreadPool` and `ThreadPoolBuilder` types
/// that form the core of the thread pool functionality.
pub mod pool;

/// Task-related types and traits.
///
/// This module defines the `Task` and `TaskHandle` types which are used
/// to represent and manage asynchronous tasks in the thread pool.
pub mod task;

/// Worker thread implementation.
///
/// This module contains the internal worker implementation that processes
/// tasks in the thread pool.
pub mod worker;

// Re-export commonly used types
pub use batch::BatchExecutor;
pub use config::ThreadPoolConfig;
pub use error::{ThreadPoolError, ThreadPoolResult};
pub use pool::{ThreadPool, ThreadPoolBuilder};
pub use task::{Task, TaskHandle};

