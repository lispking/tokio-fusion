use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_fusion::{Task, ThreadPool, ThreadPoolResult};

async fn handle_connection(mut stream: tokio::net::TcpStream) -> ThreadPoolResult<()> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    if n > 0 {
        stream.write_all(&buffer[0..n]).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ThreadPoolResult<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let thread_pool = Arc::new(ThreadPool::default());

    loop {
        let (stream, _) = listener.accept().await?;
        let task = Task::new(handle_connection(stream), 1);
        let _ = thread_pool.submit(task).await?;
    }
}
