use crate::Socket;
use crate::SocketBuilder;
use async_trait::async_trait;
use tokio::sync::Semaphore;
use tokio::sync::OwnedSemaphorePermit;
use std::sync::Arc;
use std::fmt::Debug;
use std::fmt::Display;

#[async_trait]
pub trait AsyncListener {
    async fn bind(self: Arc<Self>, permit: OwnedSemaphorePermit, 
        semaphore: Arc<Semaphore>, allocator: fn(SocketBuilder) -> Arc<dyn Socket>);
}

pub trait Listener: AsyncListener + Send + Sync + Debug + Display + 'static {
    fn close(self: Arc<Self>);
}