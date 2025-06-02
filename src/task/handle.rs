use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::sync::oneshot;

use crate::error::{ThreadPoolError, ThreadPoolResult};

/// A handle to a task that has been submitted to the thread pool
pub struct TaskHandle<T> {
    pub(crate) receiver: oneshot::Receiver<ThreadPoolResult<T>>,
}

impl<T> TaskHandle<T> {
    /// Wait for the task to complete and get the result
    pub async fn await_result(self) -> ThreadPoolResult<T> {
        self.receiver
            .await
            .unwrap_or(Err(ThreadPoolError::TaskCancelled))
    }
}

impl<T> Future for TaskHandle<T> {
    type Output = ThreadPoolResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.receiver).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(ThreadPoolError::TaskCancelled)),
            Poll::Pending => Poll::Pending,
        }
    }
}
