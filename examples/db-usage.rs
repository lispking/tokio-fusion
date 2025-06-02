use futures::StreamExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    fs as tokio_fs,
    io::AsyncWriteExt,
    sync::Mutex,
    time::{sleep, timeout},
};
use tokio_fusion::{BatchExecutor, Task, ThreadPoolBuilder, ThreadPoolError, ThreadPoolResult};
use tracing::{Level, error, info};

// Simulated database connection
#[derive(Debug, Clone)]
struct Database {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Database {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn insert(&self, key: String, value: String) -> ThreadPoolResult<()> {
        let mut data = self.data.lock().await;
        data.insert(key, value);
        // Simulate database latency
        sleep(Duration::from_millis(50)).await;
        Ok(())
    }
}

// Data structure for our examples
#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserData {
    id: usize,
    username: String,
    email: String,
    signup_timestamp: i64,
}

async fn fetch_user_data(user_id: usize) -> ThreadPoolResult<UserData> {
    // Simulate API call delay
    let delay = rand::rng().random_range(50..200);
    sleep(Duration::from_millis(delay)).await;

    // Simulate occasional failures (10% chance)
    if rand::rng().random_bool(0.1) {
        return Err(ThreadPoolError::TaskExecutionFailed(
            "API timeout".to_string(),
        ));
    }

    Ok(UserData {
        id: user_id,
        username: format!("user_{}", user_id),
        email: format!("user_{}@example.com", user_id),
        signup_timestamp: chrono::Utc::now().timestamp(),
    })
}

async fn process_user_data(user: UserData, db: Database) -> ThreadPoolResult<usize> {
    // Simulate processing time with thread-local RNG
    let delay = rand::rng().random_range(20..100);
    sleep(Duration::from_millis(delay)).await;

    // Store in database
    let user_json = serde_json::to_string(&user)?;
    db.insert(user.id.to_string(), user_json).await?;

    // Return the user ID as confirmation
    Ok(user.id)
}

async fn write_to_log_file(filename: &str, content: &str) -> ThreadPoolResult<()> {
    let mut file = tokio_fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .await
        .map_err(|e| ThreadPoolError::TaskExecutionFailed(e.to_string()))?;

    file.write_all(content.as_bytes())
        .await
        .map_err(|e| ThreadPoolError::TaskExecutionFailed(e.to_string()))?;

    file.write_all(b"\n")
        .await
        .map_err(|e| ThreadPoolError::TaskExecutionFailed(e.to_string()))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting tokio-fusion thread pool example");

    // Create a thread pool with custom configuration
    let thread_pool = Arc::new(
        ThreadPoolBuilder::new()
            .worker_threads(4)
            .queue_capacity(1000)
            .build(),
    );

    info!(
        "Thread pool created with {} worker threads",
        thread_pool.config().worker_threads
    );

    // Initialize shared resources
    let db = Database::new();
    let log_file = "activity.log";

    // Clean up log file if it exists
    let _ = tokio::fs::remove_file(log_file).await;

    // Example 1: Basic task execution with different priorities
    info!(">>> Example 1: Basic task execution with different priorities");

    // High priority task (lower number = higher priority)
    let high_pri_task = Task::new(
        async {
            info!("High priority task started");
            sleep(Duration::from_millis(200)).await;
            info!("High priority task completed");
            Ok::<_, ThreadPoolError>("High priority completed".to_string())
        },
        1, // High priority
    );

    // Medium priority task
    let med_pri_task = Task::new(
        async {
            info!("Medium priority task started");
            sleep(Duration::from_millis(100)).await;
            info!("Medium priority task completed");
            Ok::<_, ThreadPoolError>("Medium priority completed".to_string())
        },
        5, // Medium priority
    );

    // Submit tasks
    let high_handle = thread_pool.submit(high_pri_task).await?;
    let med_handle = thread_pool.submit(med_pri_task).await?;

    // Wait for results
    let high_result = high_handle.await_result().await?;
    let med_result = med_handle.await_result().await?;

    info!("High priority result: {:?}", high_result);
    info!("Medium priority result: {:?}", med_result);

    // Example 2: Batch processing of user data with retries
    info!(">>> Example 2: Batch processing of user data with retries");

    // Create a batch of user IDs to process
    let user_ids = (1..=10).collect::<Vec<_>>();
    let mut batch = BatchExecutor::new(Arc::clone(&thread_pool));

    // Add fetch tasks to the batch
    for &user_id in &user_ids {
        batch.add_task(fetch_user_data(user_id), 5); // Medium priority
    }

    // Process results with retries for failed tasks
    let max_retries = 2;
    let mut retry_count = 0;
    let mut results = Vec::new();
    let mut failed_ids = Vec::new();

    // Initial execution
    let mut stream = batch.execute_stream();
    while let Some(result) = stream.next().await {
        match result {
            Ok(user) => {
                results.push(user);
            }
            Err(e) => {
                error!("Failed to fetch user data: {}", e);
                failed_ids.push(user_ids[results.len() + failed_ids.len()]);
            }
        }
    }

    // Retry logic
    while !failed_ids.is_empty() && retry_count < max_retries {
        retry_count += 1;
        info!(
            "Retry {} for {} failed tasks",
            retry_count,
            failed_ids.len()
        );

        let mut retry_batch = BatchExecutor::new(Arc::clone(&thread_pool));
        for &id in &failed_ids {
            retry_batch.add_task(fetch_user_data(id), 1); // Higher priority for retries
        }

        let mut new_failed = Vec::new();
        let mut stream = retry_batch.execute_stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(user) => {
                    results.push(user);
                }
                Err(e) => {
                    error!("Retry {} failed: {}", retry_count, e);
                    new_failed.push(failed_ids[results.len() - 1]);
                }
            }
        }

        failed_ids = new_failed;
    }

    info!(
        "Successfully processed {}/{} users",
        results.len(),
        user_ids.len()
    );

    // Example 3: Parallel database operations with logging
    info!(">>> Example 3: Parallel database operations with logging");

    // Process users in parallel with logging
    let mut process_batch = BatchExecutor::new(Arc::clone(&thread_pool));
    let db_clone = db.clone();

    for user in results {
        let user_clone = user.clone();
        let db_clone = db_clone.clone();
        let log_file = log_file.to_string();

        process_batch.add_task(
            async move {
                // Process user data
                let user_id = process_user_data(user_clone, db_clone).await?;

                // Log the operation
                let log_entry = format!("[{}] Processed user {}", chrono::Local::now(), user_id);
                write_to_log_file(&log_file, &log_entry).await?;

                Ok::<_, ThreadPoolError>(user_id)
            },
            5, // Medium priority
        );
    }

    // Process with a timeout
    let process_results = match timeout(Duration::from_secs(5), process_batch.execute()).await {
        Ok(results) => results,
        Err(_) => {
            error!("Processing timed out after 5 seconds");
            return Ok(());
        }
    };

    // Log final status
    let success_count = process_results.iter().filter(|r| r.is_ok()).count();
    let error_count = process_results.len() - success_count;
    info!(
        "Database operations completed: {} success, {} errors",
        success_count, error_count
    );

    // Example 4: Batch processing with rate limiting
    info!(">>> Example 4: Batch processing with rate limiting");

    // Process a batch of items with a simple rate limit
    let batch_size = 5;
    let delay_between_batches = Duration::from_secs(1);

    for chunk in (1..=15).collect::<Vec<_>>().chunks(batch_size) {
        let mut batch = BatchExecutor::new(Arc::clone(&thread_pool));

        for &i in chunk {
            let db_clone = db.clone();
            let log_file = log_file.to_string();

            batch.add_task(
                async move {
                    // Simulate work with thread-local RNG
                    let delay = {
                        let mut rng = rand::rng();
                        rng.random_range(50..200)
                    };
                    sleep(Duration::from_millis(delay)).await;

                    // Store in database
                    let data = format!("data_{:04}", i);
                    db_clone.insert(i.to_string(), data).await?;

                    // Log the operation
                    let log_entry = format!("[{}] Processed data {}", chrono::Local::now(), i);
                    write_to_log_file(&log_file, &log_entry).await?;

                    Ok::<_, ThreadPoolError>(i)
                },
                5, // Medium priority
            );
        }

        // Execute the batch
        let results = batch.execute().await;
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        info!(
            "Processed batch: {}/{} tasks succeeded",
            success_count,
            results.len()
        );

        // Rate limiting
        if chunk.len() == batch_size {
            sleep(delay_between_batches).await;
        }
    }

    // Example 5: Error handling and cleanup
    info!(">>> Example 5: Error handling and cleanup");

    // Create tasks that might fail
    let error_task = Task::new(
        async {
            if rand::rng().random_bool(0.7) {
                Err(ThreadPoolError::TaskExecutionFailed(
                    "Random failure".to_string(),
                ))
            } else {
                Ok("Success!".to_string())
            }
        },
        5,
    );

    // Submit with timeout
    let handle = thread_pool.submit(error_task).await?;
    match timeout(Duration::from_secs(2), handle.await_result()).await {
        Ok(Ok(result)) => info!("Task succeeded: {}", result),
        Ok(Err(e)) => error!("Task failed: {}", e),
        Err(_) => error!("Task timed out"),
    }

    // Clean up resources
    info!("Cleaning up resources...");

    // Read and display the log file
    info!("Log file contents:");
    if let Ok(contents) = tokio_fs::read_to_string(log_file).await {
        for line in contents.lines() {
            info!("  {}", line);
        }
    }

    // Clean up log file
    let _ = tokio::fs::remove_file(log_file).await;

    info!("All examples completed successfully!");
    Ok(())
}
