use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tokio_fusion::{Task, ThreadPool, ThreadPoolResult};

async fn backup_data() -> ThreadPoolResult<()> {
    println!("Data backup completed");
    Ok(())
}

#[tokio::main]
async fn main() -> ThreadPoolResult<()> {
    let thread_pool = Arc::new(ThreadPool::default());
    let interval = Duration::from_secs(3600);

    loop {
        let task = Task::new(backup_data(), 1);
        let _ = thread_pool.submit(task).await?;
        sleep(interval).await;
    }
}
