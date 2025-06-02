use std::sync::Arc;
use tokio_fusion::{BatchExecutor, ThreadPool, ThreadPoolResult};

async fn fetch_data(source: &str) -> ThreadPoolResult<String> {
    Ok(format!("Data from {}", source))
}

#[tokio::main]
async fn main() {
    let thread_pool = Arc::new(ThreadPool::default());
    let mut batch = BatchExecutor::new(Arc::clone(&thread_pool));

    let sources = vec!["source1", "source2", "source3"];
    for source in sources {
        batch.add_task(fetch_data(source), 1);
    }

    let results = batch.execute().await;
    for result in results {
        println!("{:?}", result);
    }
}
