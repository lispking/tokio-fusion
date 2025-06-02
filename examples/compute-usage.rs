use std::sync::Arc;
use tokio_fusion::{BatchExecutor, ThreadPool, ThreadPoolResult};

async fn compute(data: u64) -> ThreadPoolResult<u64> {
    let result = data * data;
    Ok(result)
}

#[tokio::main]
async fn main() {
    let thread_pool = Arc::new(ThreadPool::default());
    let mut batch = BatchExecutor::new(Arc::clone(&thread_pool));

    let data = vec![1, 2, 3, 4, 5];
    for d in data {
        batch.add_task(compute(d), 1);
    }

    let results = batch.execute().await;
    for result in results {
        println!("{:?}", result);
    }
}
